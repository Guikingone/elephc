//! Purpose:
//! Synthesises the built-in reflection class checker metadata so user code can
//! receive `ReflectionAttribute` instances and query class/member attributes,
//! class-shape facts (`isAbstract`/`isSubclassOf`/`hasMethod`/`getConstants`/…),
//! and modifier bitmasks through a small PHP-compatible Reflection surface.
//!
//! Called from:
//! - `crate::types::checker::driver::init` (alongside `inject_builtin_throwables`).
//!
//! Key details:
//! - Property and method bodies are dummies, simple private-slot accessors, or
//!   generic bodies (`in_array`/`strtolower`/`foreach`) reading a private slot;
//!   runtime population of those slots is handled by codegen-only reflection
//!   constructors (see `crate::codegen_ir::lower_inst::objects::reflection`).
//! - Every core Reflection* shell implements `Reflector` (which extends
//!   `Stringable`); their `__toString()` throws rather than fabricating PHP's
//!   object-dump text (`builtin_reflection_unsupported_tostring_method`).
//! - A new `array`-typed metadata slot MUST use `str_array_type()`/
//!   `mixed_array_type()`, not the bare `array_type()` — the bare `array`
//!   shape defaults its element type to `mixed` under gradual typing, which
//!   mismatches the plain-string/plain-Mixed runtime layout the EIR bakers
//!   produce and crashes element reads (`in_array()`, etc.).

use std::collections::{HashMap, HashSet};

use crate::errors::CompileError;
use crate::names::{php_symbol_key, Name};
use crate::parser::ast::{
    BinOp, ClassConst, ClassMethod, ClassProperty, Expr, ExprKind, Stmt, StmtKind, TypeExpr,
    Visibility,
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
            constants: vec![ClassConst {
                name: "IS_INSTANCEOF".to_string(),
                visibility: Visibility::Public,
                is_final: false,
                // PHP: ReflectionAttribute::IS_INSTANCEOF = 2 — the getAttributes() flag that
                // matches attributes assignable to (instanceof) the given name.
                value: Expr::new(ExprKind::IntLiteral(2), crate::span::Span::dummy()),
                span: crate::span::Span::dummy(),
                attributes: Vec::new(),
            }],
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
        // PHP: public readonly `string $name` / `string $class`. Recognition-level
        // stubs (un-backed, default `''`) so `$rm->name` / `$rm->class` type-check;
        // runtime population is a separate follow-up.
        reflection_method
            .properties
            .push(builtin_property("name", Visibility::Public, Some(TypeExpr::Str), empty_string()));
        reflection_method
            .properties
            .push(builtin_property("class", Visibility::Public, Some(TypeExpr::Str), empty_string()));
        // Real PHP modifier bitmask (IS_STATIC|IS_PUBLIC|IS_PROTECTED|IS_PRIVATE|
        // IS_ABSTRACT|IS_FINAL), baked at construction from the reflected
        // method's actual visibility/staticness/abstractness — see
        // `emit_reflection_method_modifiers` in the EIR codegen.
        reflection_method
            .properties
            .push(builtin_property("__modifiers", Visibility::Private, Some(TypeExpr::Int), int_lit(0)));
        reflection_method.implements.push("Reflector".to_string());
        // PHP: `getName(): string` — slot getter on `__name`.
        reflection_method
            .methods
            .push(builtin_reflection_slot_getter("getName", "__name", TypeExpr::Str));
        // PHP: `getShortName(): string` — methods are never namespaced, so
        // PHP's short name for a method is always identical to its full name
        // (verified: `(new ReflectionMethod($c, $m))->getShortName() ===
        // (new ReflectionMethod($c, $m))->getName()`); delegate rather than
        // duplicating the `__name` slot.
        reflection_method.methods.push(builtin_reflection_computed_method(
            "getShortName",
            TypeExpr::Str,
            Expr::new(
                ExprKind::MethodCall {
                    object: Box::new(Expr::new(ExprKind::This, crate::span::Span::dummy())),
                    method: "getName".to_string(),
                    args: Vec::new(),
                },
                crate::span::Span::dummy(),
            ),
        ));
        // PHP: `getModifiers(): int` — slot getter on the baked bitmask.
        reflection_method
            .methods
            .push(builtin_reflection_slot_getter("getModifiers", "__modifiers", TypeExpr::Int));
        // Real visibility/staticness/abstractness checks — single-bit tests
        // against `__modifiers` (php -n verified bit values: IS_PUBLIC=1,
        // IS_PROTECTED=2, IS_STATIC=16, IS_ABSTRACT=64).
        reflection_method.methods.push(builtin_reflection_computed_method(
            "isPublic",
            TypeExpr::Bool,
            modifier_bit_test_expr("__modifiers", 1),
        ));
        reflection_method.methods.push(builtin_reflection_computed_method(
            "isProtected",
            TypeExpr::Bool,
            modifier_bit_test_expr("__modifiers", 2),
        ));
        reflection_method.methods.push(builtin_reflection_computed_method(
            "isStatic",
            TypeExpr::Bool,
            modifier_bit_test_expr("__modifiers", 16),
        ));
        reflection_method.methods.push(builtin_reflection_computed_method(
            "isAbstract",
            TypeExpr::Bool,
            modifier_bit_test_expr("__modifiers", 64),
        ));
        // php -n verified bit values: IS_PRIVATE=4, IS_FINAL=32. Added alongside
        // `isPublic`/`isProtected`/`isStatic`/`isAbstract` above (J4: the flat method-table
        // dynamic-construction feature needs the full visibility-bit surface).
        reflection_method.methods.push(builtin_reflection_computed_method(
            "isPrivate",
            TypeExpr::Bool,
            modifier_bit_test_expr("__modifiers", 4),
        ));
        reflection_method.methods.push(builtin_reflection_computed_method(
            "isFinal",
            TypeExpr::Bool,
            modifier_bit_test_expr("__modifiers", 32),
        ));
        reflection_method
            .methods
            .push(builtin_reflection_unsupported_tostring_method("ReflectionMethod"));
        // Declaring source file (empty-string sentinel for "unknown"), baked at construction
        // from the OWNER class's `__file` (a method's `getFileName()` equals its class's, per
        // PHP). Backs `getFileName()`.
        reflection_method
            .properties
            .push(builtin_property("__file", Visibility::Private, Some(TypeExpr::Str), empty_string()));
        reflection_method
            .methods
            .push(builtin_reflection_get_file_name_method());
        // PHP: `getClosure(?object $object = null): ?Closure` — un-backed stub
        // returning `null` typed `mixed`. The optional `object` param (modelled
        // `mixed`) lets `$method->getClosure($obj)` type-check; closure/object
        // return → `mixed` for the EIR-lowering reason documented on
        // `builtin_reflection_property`.
        reflection_method
            .methods
            .push(builtin_reflection_literal_method_with_params(
                "getClosure",
                mixed_type(),
                null_lit(),
                vec![("object", Some(mixed_type()), null_lit(), false)],
            ));
        // PHP: `getDeclaringClass(): ReflectionClass` — un-backed stub returning
        // `null` typed `mixed` (object return → mixed; see
        // `builtin_reflection_property` for the EIR-lowering reason).
        reflection_method
            .methods
            .push(builtin_reflection_literal_method("getDeclaringClass", mixed_type(), null_lit()));
        // PHP class constants — php -n verified:
        // `ReflectionMethod::IS_STATIC=16, IS_PUBLIC=1, IS_PROTECTED=2,
        // IS_PRIVATE=4, IS_ABSTRACT=64, IS_FINAL=32`.
        for (name, value) in [
            ("IS_STATIC", 16),
            ("IS_PUBLIC", 1),
            ("IS_PROTECTED", 2),
            ("IS_PRIVATE", 4),
            ("IS_ABSTRACT", 64),
            ("IS_FINAL", 32),
        ] {
            reflection_method.constants.push(ClassConst {
                name: name.to_string(),
                visibility: Visibility::Public,
                is_final: false,
                value: Expr::new(ExprKind::IntLiteral(value), crate::span::Span::dummy()),
                span: crate::span::Span::dummy(),
                attributes: Vec::new(),
            });
        }
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

/// Returns a `TypeExpr` for `array<string>` (an indexed array of strings).
/// Used for the construction-baked metadata slots this feature adds
/// (`__ancestors_lower`, `__interfaces`, …): the bare `array` shape (see
/// `array_type()`) defaults its element type to `mixed` under gradual
/// typing, which does not match the plain-string-element runtime layout
/// `emit_string_array` bakes — `in_array()`/element reads on a
/// `mixed`-declared-but-string-shaped array crash by treating each element
/// as a boxed Mixed cell it never was. Declaring the real element type keeps
/// the static type and runtime representation in sync.
fn str_array_type() -> TypeExpr {
    TypeExpr::Array(Box::new(TypeExpr::Str))
}

/// Returns a `TypeExpr` for `array<mixed>`. Used for `__const_values`, which
/// `emit_mixed_array` bakes as boxed Mixed cells — see `str_array_type()`
/// for why the element type must match the actual runtime representation.
fn mixed_array_type() -> TypeExpr {
    TypeExpr::Array(Box::new(mixed_type()))
}

/// Returns a bare `$name` variable-reference expression.
fn var_expr(name: &str) -> Expr {
    Expr::new(ExprKind::Variable(name.to_string()), crate::span::Span::dummy())
}

/// Returns a `$this->property` access expression.
fn this_prop_expr(property: &str) -> Expr {
    Expr::new(
        ExprKind::PropertyAccess {
            object: Box::new(Expr::new(ExprKind::This, crate::span::Span::dummy())),
            property: property.to_string(),
        },
        crate::span::Span::dummy(),
    )
}

/// Returns a call to the free function `name` with the given positional
/// argument expressions.
fn free_call_expr(name: &str, args: Vec<Expr>) -> Expr {
    Expr::new(
        ExprKind::FunctionCall {
            name: Name::unqualified(name),
            args,
        },
        crate::span::Span::dummy(),
    )
}

/// Returns a public no-arg method whose body is a single `return EXPR;`
/// where `EXPR` is computed by `body_expr` (a general-purpose alternative to
/// `builtin_reflection_slot_getter`/`builtin_reflection_literal_method` for
/// bodies that combine multiple slots or call builtin functions instead of
/// surfacing one property verbatim).
fn builtin_reflection_computed_method(
    method_name: &str,
    return_type: TypeExpr,
    body_expr: Expr,
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
        body: vec![Stmt::new(StmtKind::Return(Some(body_expr)), dummy_span)],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public single-`string`-parameter method whose body is a single
/// `return EXPR;`, where `EXPR` is built by `body_expr` from the parameter
/// name. Used for `hasMethod`/`hasProperty`/`isSubclassOf`/
/// `implementsInterface`: each does a case-appropriate membership test
/// against a private array slot baked at construction time (see
/// `emit_reflection_*` in the EIR codegen), so the SAME generic method body
/// works correctly for any runtime argument value, not just compile-time
/// literals.
fn builtin_reflection_string_arg_method(
    method_name: &str,
    param_name: &str,
    return_type: TypeExpr,
    body_expr: Expr,
) -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: method_name.to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![(param_name.to_string(), Some(TypeExpr::Str), None, false)],
        variadic: None,
        variadic_type: None,
        return_type: Some(return_type),
        by_ref_return: false,
        body: vec![Stmt::new(StmtKind::Return(Some(body_expr)), dummy_span)],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns `in_array(strtolower(ltrim($param, '\\')), $this->haystack_property, true)`:
/// a case-insensitive, leading-backslash-tolerant membership test against a
/// private array property. Used for `isSubclassOf`/`implementsInterface`,
/// which PHP resolves case-insensitively (class/interface names are
/// case-insensitive identifiers).
fn case_insensitive_membership_expr(param_name: &str, haystack_property: &str) -> Expr {
    let trimmed = free_call_expr(
        "ltrim",
        vec![var_expr(param_name), Expr::new(ExprKind::StringLiteral("\\".to_string()), crate::span::Span::dummy())],
    );
    let folded = free_call_expr("strtolower", vec![trimmed]);
    free_call_expr(
        "in_array",
        vec![folded, this_prop_expr(haystack_property), Expr::new(ExprKind::BoolLiteral(true), crate::span::Span::dummy())],
    )
}

/// Returns `in_array(strtolower($param), $this->haystack_property, true)`: a
/// case-insensitive membership test against a private array property. Used
/// for `hasMethod`, which PHP resolves case-insensitively but never strips a
/// leading backslash (method names are never namespaced).
fn case_insensitive_method_membership_expr(param_name: &str, haystack_property: &str) -> Expr {
    let folded = free_call_expr("strtolower", vec![var_expr(param_name)]);
    free_call_expr(
        "in_array",
        vec![folded, this_prop_expr(haystack_property), Expr::new(ExprKind::BoolLiteral(true), crate::span::Span::dummy())],
    )
}

/// Returns `in_array($param, $this->haystack_property, true)`: an exact-case
/// membership test against a private array property. Used for `hasProperty`,
/// since PHP property names are case-SENSITIVE (unlike class/method names).
fn exact_membership_expr(param_name: &str, haystack_property: &str) -> Expr {
    free_call_expr(
        "in_array",
        vec![var_expr(param_name), this_prop_expr(haystack_property), Expr::new(ExprKind::BoolLiteral(true), crate::span::Span::dummy())],
    )
}

/// Returns a `TypeExpr` for `string|false` (PHP's `getFileName()`/`getParentClass()`-shaped
/// "string result, or `false` when unavailable" return contract).
fn str_or_false_type() -> TypeExpr {
    TypeExpr::Union(vec![TypeExpr::Str, TypeExpr::Bool])
}

/// Returns `$this->slot === '' ? false : $this->slot`: the shared "empty-string sentinel means
/// PHP `false`" pattern baked slots use for optional string metadata (`__file`'s declaring path,
/// `__parent_name`'s resolved parent). The slot is populated at codegen time (empty string when
/// the real value is unknown/absent); this body never needs to know WHY it is empty.
fn empty_string_sentinel_expr(slot: &str) -> Expr {
    let dummy_span = crate::span::Span::dummy();
    Expr::new(
        ExprKind::Ternary {
            condition: Box::new(Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(this_prop_expr(slot)),
                    op: BinOp::StrictEq,
                    right: Box::new(Expr::new(ExprKind::StringLiteral(String::new()), dummy_span)),
                },
                dummy_span,
            )),
            then_expr: Box::new(Expr::new(ExprKind::BoolLiteral(false), dummy_span)),
            else_expr: Box::new(this_prop_expr(slot)),
        },
        dummy_span,
    )
}

/// Returns a public no-arg method whose body is `if ($this->guard_property) { throw new
/// ReflectionException(message); } return body_expr;`.
///
/// Used to gate `ReflectionFunction::getName()`/`getShortName()`/`getFileName()` on
/// closure-backed instances constructed from a closure LITERAL (`new
/// ReflectionFunction(function ($x) {...})`): PHP's real closure name embeds the
/// declaring file (or enclosing function/method) and line
/// (`"{closure:FILE:LINE}"`/`"{closure:Class::method():LINE}"`, php -n VERIFIED on
/// PHP 8.5 — NOT the bare `"{closure}"` used by older PHP versions), which elephc has
/// no per-closure source-location tracking to reproduce soundly. Rather than bake a
/// value that would silently mismatch real PHP, these methods THROW at runtime for a
/// closure-literal-backed instance (see `crate::codegen_ir::lower_inst::objects::reflection`
/// for where `__unbacked_name` is set to `true` only for that construction path — string-
/// literal and first-class-callable constructions leave it `false` and stay fully backed).
fn builtin_reflection_guarded_method(
    method_name: &str,
    return_type: TypeExpr,
    guard_property: &str,
    message: &str,
    body_expr: Expr,
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
        body: vec![
            Stmt::new(
                StmtKind::If {
                    condition: this_prop_expr(guard_property),
                    then_body: vec![Stmt::new(
                        StmtKind::Throw(Expr::new(
                            ExprKind::NewObject {
                                class_name: Name::unqualified("ReflectionException"),
                                args: vec![Expr::new(
                                    ExprKind::StringLiteral(message.to_string()),
                                    dummy_span,
                                )],
                            },
                            dummy_span,
                        )),
                        dummy_span,
                    )],
                    elseif_clauses: Vec::new(),
                    else_body: None,
                },
                dummy_span,
            ),
            Stmt::new(StmtKind::Return(Some(body_expr)), dummy_span),
        ],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public no-arg `getFileName(): string|false` method reading the private `__file`
/// slot baked at construction time (see `crate::codegen_ir::lower_inst::objects::reflection`):
/// the declaring source file's path when known, PHP's `false` sentinel otherwise (an internal/
/// builtin reflected symbol, or one this snapshot could not attribute — see
/// `crate::pipeline::scan_reflection_source_files`). Shared by every reflection owner EXCEPT
/// `ReflectionFunction`, which uses the guarded `builtin_reflection_function_get_file_name_method`
/// instead (only `ReflectionFunction` instances can be closure-backed).
fn builtin_reflection_get_file_name_method() -> ClassMethod {
    builtin_reflection_computed_method("getFileName", str_or_false_type(), empty_string_sentinel_expr("__file"))
}

/// `ReflectionFunction`-only `getFileName()`: same `__file` slot as
/// `builtin_reflection_get_file_name_method`, but gated on `__unbacked_file` — a closure-literal-
/// backed instance (elephc cannot reproduce PHP's per-closure declaring-file tracking) OR a
/// dynamic descriptor-based instance (M2 PART A: no source file is tracked for ANY dynamically
/// reflected value either — see `crate::codegen_ir::lower_inst::objects::reflection_function_dynamic`).
/// `__unbacked_file` is a SEPARATE flag from `__unbacked_name` (JURY ADDENDUM item 2): the dynamic
/// path sets `__unbacked_name = false` (so `getName()`/`getShortName()` stay backed, returning
/// `"{closure}"` or the resolved real name) while STILL setting `__unbacked_file = true` (so
/// `getFileName()`/`getStartLine()` keep throwing) — a closure-literal instance sets BOTH flags
/// `true`, matching its pre-existing behavior unchanged.
fn builtin_reflection_function_get_file_name_method() -> ClassMethod {
    builtin_reflection_guarded_method(
        "getFileName",
        str_or_false_type(),
        "__unbacked_file",
        "ReflectionFunction::getFileName() is not supported for this instance: elephc does not track a declaring source file for a closure-literal-backed or dynamically-reflected instance",
        empty_string_sentinel_expr("__file"),
    )
}

/// Returns a public `getParentClass(): ReflectionClass|false` method: PHP's `false` when the
/// reflected class has no parent (the baked `__parent_name` slot is the empty-string sentinel),
/// otherwise a freshly constructed `ReflectionClass` for the parent. Reuses the SAME dynamic-name
/// construction path a `new ReflectionClass($runtimeString)` call already takes for a non-literal
/// `Str`-typed argument (see
/// `crate::codegen_ir::lower_inst::objects::reflection::lower_reflection_class_new_dynamic`), so
/// this single PHP-level body correctly serves both a literal-constructed and a
/// dynamically-constructed receiver: `__parent_name` is always read from a slot at runtime either
/// way, never known at THIS method's own compile time.
fn builtin_reflection_get_parent_class_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    let return_type = TypeExpr::Union(vec![
        TypeExpr::Named(Name::unqualified("ReflectionClass")),
        TypeExpr::Bool,
    ]);
    let body_expr = Expr::new(
        ExprKind::Ternary {
            condition: Box::new(Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(this_prop_expr("__parent_name")),
                    op: BinOp::StrictEq,
                    right: Box::new(Expr::new(ExprKind::StringLiteral(String::new()), dummy_span)),
                },
                dummy_span,
            )),
            then_expr: Box::new(Expr::new(ExprKind::BoolLiteral(false), dummy_span)),
            else_expr: Box::new(Expr::new(
                ExprKind::NewObject {
                    class_name: Name::unqualified("ReflectionClass"),
                    args: vec![this_prop_expr("__parent_name")],
                },
                dummy_span,
            )),
        },
        dummy_span,
    );
    builtin_reflection_computed_method("getParentClass", return_type, body_expr)
}

/// Returns a public `__toString(): string` method that unconditionally
/// throws. `Reflector` (which every core Reflection* shell implements)
/// extends `Stringable`, so a concrete, non-abstract class implementing it
/// must supply a `__toString()` body to satisfy the interface contract.
/// elephc does not model PHP's real `Reflection*::__toString()` object-dump
/// text, so — per the "no stub" policy — the body throws a real `\Error`
/// instead of fabricating output; echoing a Reflection object stays a loud,
/// observable failure rather than silently returning an empty string.
fn builtin_reflection_unsupported_tostring_method(class_name: &str) -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    let message = format!(
        "{}::__toString() is not supported: elephc's reflection shim does not implement PHP's object-dump text",
        class_name
    );
    ClassMethod {
        name: "__toString".to_string(),
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
            StmtKind::Throw(Expr::new(
                ExprKind::NewObject {
                    class_name: Name::unqualified("Error"),
                    args: vec![Expr::new(ExprKind::StringLiteral(message), dummy_span)],
                },
                dummy_span,
            )),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns a public method (optionally with regular params and/or a variadic tail) whose body
/// unconditionally throws a `ReflectionException(message)` — never returns. Used for methods this
/// shell cannot back for ANY construction path yet (`getStartLine`'s missing declaration-line
/// tracking) or cannot back SOUNDLY for the dynamic descriptor-based construction path
/// (`invoke`/`invokeArgs`: no wiring yet through the uniform closure invoker; `getParameters`'s
/// per-parameter runtime array is deferred — see
/// `crate::codegen_ir::lower_inst::objects::reflection_function_dynamic`'s module doc). Mirrors
/// `builtin_reflection_guarded_method`'s throw shape but skips the `if` guard entirely (M2 PART A,
/// JURY ADDENDUM item 4: "ALL unbacked ReflectionFunction/ReflectionParameter methods guarded with
/// catchable ReflectionException — never partial objects"): the body never returns a value, so no
/// declared `return_type` is ever silently violated either.
fn builtin_reflection_unconditional_throw_method(
    method_name: &str,
    return_type: TypeExpr,
    message: &str,
    params: Vec<(&str, Option<TypeExpr>, Option<Expr>, bool)>,
    variadic: Option<(&str, TypeExpr)>,
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
        variadic: variadic.as_ref().map(|(name, _)| name.to_string()),
        variadic_type: variadic.map(|(_, ty)| ty),
        return_type: Some(return_type),
        by_ref_return: false,
        body: vec![Stmt::new(
            StmtKind::Throw(Expr::new(
                ExprKind::NewObject {
                    class_name: Name::unqualified("ReflectionException"),
                    args: vec![Expr::new(ExprKind::StringLiteral(message.to_string()), dummy_span)],
                },
                dummy_span,
            )),
            dummy_span,
        )],
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Returns `($this->modifiers_property & mask) !== 0`: a single-bit test
/// against a private `int` modifiers slot baked at construction time. Used
/// for `isPublic`/`isStatic`/`isProtected`/`isAbstract` on `ReflectionMethod`.
fn modifier_bit_test_expr(modifiers_property: &str, mask: i64) -> Expr {
    let dummy_span = crate::span::Span::dummy();
    Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(this_prop_expr(modifiers_property)),
                    op: BinOp::BitAnd,
                    right: Box::new(Expr::new(ExprKind::IntLiteral(mask), dummy_span)),
                },
                dummy_span,
            )),
            op: BinOp::NotEq,
            right: Box::new(Expr::new(ExprKind::IntLiteral(0), dummy_span)),
        },
        dummy_span,
    )
}

/// Returns a public `getMethods(int $filter = 0): array` / `getProperties(int $filter = 0):
/// array` method (K1 Part A) whose body is `get_members_body(names_property, member_class)`.
fn get_members_method(method_name: &str, names_property: &str, member_class: &str) -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: method_name.to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![("filter".to_string(), Some(TypeExpr::Int), int_lit(0), false)],
        variadic: None,
        variadic_type: None,
        return_type: Some(array_type()),
        by_ref_return: false,
        body: get_members_body(names_property, member_class),
        span: dummy_span,
        attributes: Vec::new(),
    }
}

/// Builds the shared `getMethods([$filter]): array<ReflectionMethod>` /
/// `getProperties([$filter]): array<ReflectionProperty>` body: iterates
/// `$this->names_property` (already PHP declaration-order and parent-private-excluded — see
/// `crate::codegen::runtime::data::reflect_member_registry::method_decl_order_and_names`/
/// `property_decl_order_and_names`, which bake it at construction time), constructs a
/// `member_class` shell for each visible name via the EXISTING dynamic constructor
/// (`new ReflectionMethod($this->__name, $name)` / `new ReflectionProperty(...)` — the J4
/// flat-registry dispatcher, which soundly serves both a literal- and a
/// dynamically-constructed `$this` since `$this->__name` is always a runtime string), and
/// keeps only the ones matching `$filter`: `0` means no filtering (PHP's real no-arg/zero
/// behavior — php -n verified), otherwise PHP's OR-bitmask semantics
/// `($m->getModifiers() & $filter) !== 0`.
fn get_members_body(names_property: &str, member_class: &str) -> Vec<Stmt> {
    let dummy_span = crate::span::Span::dummy();
    let filter_is_zero = Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(var_expr("filter")),
            op: BinOp::StrictEq,
            right: Box::new(Expr::new(ExprKind::IntLiteral(0), dummy_span)),
        },
        dummy_span,
    );
    let modifiers_match_filter = Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(Expr::new(
                        ExprKind::MethodCall {
                            object: Box::new(var_expr("__m")),
                            method: "getModifiers".to_string(),
                            args: Vec::new(),
                        },
                        dummy_span,
                    )),
                    op: BinOp::BitAnd,
                    right: Box::new(var_expr("filter")),
                },
                dummy_span,
            )),
            op: BinOp::StrictNotEq,
            right: Box::new(Expr::new(ExprKind::IntLiteral(0), dummy_span)),
        },
        dummy_span,
    );
    vec![
        Stmt::assign("__result", Expr::new(ExprKind::ArrayLiteral(Vec::new()), dummy_span)),
        Stmt::new(
            StmtKind::Foreach {
                array: this_prop_expr(names_property),
                key_var: None,
                value_var: "__n".to_string(),
                value_by_ref: false,
                body: vec![
                    Stmt::assign(
                        "__m",
                        Expr::new(
                            ExprKind::NewObject {
                                class_name: Name::unqualified(member_class),
                                args: vec![this_prop_expr("__name"), var_expr("__n")],
                            },
                            dummy_span,
                        ),
                    ),
                    Stmt::new(
                        StmtKind::If {
                            condition: Expr::new(
                                ExprKind::BinaryOp {
                                    left: Box::new(filter_is_zero.clone()),
                                    op: BinOp::Or,
                                    right: Box::new(modifiers_match_filter.clone()),
                                },
                                dummy_span,
                            ),
                            then_body: vec![Stmt::new(
                                StmtKind::ArrayPush {
                                    array: "__result".to_string(),
                                    value: var_expr("__m"),
                                },
                                dummy_span,
                            )],
                            elseif_clauses: Vec::new(),
                            else_body: None,
                        },
                        dummy_span,
                    ),
                ],
            },
            dummy_span,
        ),
        Stmt::new(StmtKind::Return(Some(var_expr("__result"))), dummy_span),
    ]
}

/// Builds the `getConstants(): array` body: iterates the parallel
/// `$this->names_property`/`$this->values_property` slots baked at
/// construction time (own + inherited class constants that fold to a
/// compile-time literal — see `fold_reflection_class_const_value` in the EIR
/// codegen) and assembles them into a fresh name-keyed associative array.
fn get_constants_body(names_property: &str, values_property: &str) -> Vec<Stmt> {
    let dummy_span = crate::span::Span::dummy();
    vec![
        Stmt::assign("__result", Expr::new(ExprKind::ArrayLiteral(Vec::new()), dummy_span)),
        Stmt::new(
            StmtKind::Foreach {
                array: this_prop_expr(names_property),
                key_var: Some("__i".to_string()),
                value_var: "__n".to_string(),
                value_by_ref: false,
                body: vec![Stmt::new(
                    StmtKind::ArrayAssign {
                        array: "__result".to_string(),
                        index: var_expr("__n"),
                        value: Expr::new(
                            ExprKind::ArrayAccess {
                                array: Box::new(this_prop_expr(values_property)),
                                index: Box::new(var_expr("__i")),
                            },
                            dummy_span,
                        ),
                    },
                    dummy_span,
                )],
            },
            dummy_span,
        ),
        Stmt::new(StmtKind::Return(Some(var_expr("__result"))), dummy_span),
    ]
}

/// Builds the `getConstant(string $name): mixed` body: linearly searches
/// `$this->names_property` for `$name` and returns the corresponding
/// `$this->values_property` entry, or PHP's documented `false` sentinel when
/// no constant with that name was baked (either genuinely undefined, or a
/// constant whose value expression wasn't a compile-time literal this
/// reflection helper can materialize — see `get_constants_body`).
fn get_constant_body(names_property: &str, values_property: &str) -> Vec<Stmt> {
    let dummy_span = crate::span::Span::dummy();
    vec![
        Stmt::new(
            StmtKind::Foreach {
                array: this_prop_expr(names_property),
                key_var: Some("__i".to_string()),
                value_var: "__n".to_string(),
                value_by_ref: false,
                body: vec![Stmt::new(
                    StmtKind::If {
                        condition: Expr::new(
                            ExprKind::BinaryOp {
                                left: Box::new(var_expr("__n")),
                                op: BinOp::StrictEq,
                                right: Box::new(var_expr("name")),
                            },
                            dummy_span,
                        ),
                        then_body: vec![Stmt::new(
                            StmtKind::Return(Some(Expr::new(
                                ExprKind::ArrayAccess {
                                    array: Box::new(this_prop_expr(values_property)),
                                    index: Box::new(var_expr("__i")),
                                },
                                dummy_span,
                            ))),
                            dummy_span,
                        )],
                        elseif_clauses: Vec::new(),
                        else_body: None,
                    },
                    dummy_span,
                )],
            },
            dummy_span,
        ),
        Stmt::new(StmtKind::Return(Some(bool_lit(false).unwrap())), dummy_span),
    ]
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

/// Returns the public `__construct(Closure|string $function)` for `ReflectionFunction`. The
/// body is empty; codegen populates the metadata slots from the reflected function/closure's
/// signature. Modeled `mixed` under gradual typing — matching the `ReflectionClass
/// __construct(object|string $objectOrClass)` precedent (see `reflection_class_literal_arg` in
/// `crate::types::checker::inference::objects::constructors`) — since elephc types closures as
/// `PhpType::Callable` (not a dedicated `Closure` object type), so a plain `Str|Callable`
/// parameter hint would already reject nothing extra; `mixed` keeps the ACTUAL string-vs-closure
/// boundary enforcement in `validate_reflection_owner_constructor`, which also rejects a
/// dynamically-typed Closure value (a `Closure`-typed variable/parameter with no statically
/// resolvable identity) that this shell cannot soundly back at all.
fn builtin_reflection_function_constructor_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "__construct".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![("function".to_string(), Some(mixed_type()), None, false)],
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
        implements: vec!["Reflector".to_string()],
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
            // PHP: public readonly `string $name`. Recognition-level stub
            // (un-backed, default `''`) so `$rf->name` type-checks; runtime
            // population is a separate follow-up.
            builtin_property("name", Visibility::Public, Some(TypeExpr::Str), empty_string()),
            // Declaring source file (empty-string sentinel for "unknown"), baked from
            // `crate::pipeline::scan_reflection_source_files`'s snapshot. Backs `getFileName()`.
            builtin_property("__file", Visibility::Private, Some(TypeExpr::Str), empty_string()),
            // `true` only when this instance was constructed from a closure LITERAL (`new
            // ReflectionFunction(function ($x) {...})`/`fn (...) => ...`): num-params-related
            // slots (`__num_params`/`__num_required`/`__params`) are still soundly derived from
            // the closure's own AST and stay backed, but `__name`/`__short`/`__file` are NOT —
            // PHP's real closure name embeds the declaring file/line (php -n verified 8.5 format:
            // `"{closure:FILE:LINE}"`/`"{closure:Class::method():LINE}"`), which elephc has no
            // per-closure source-location tracking to reproduce. `getName`/`getShortName`/
            // `getFileName` throw instead of faking a value when this is `true` (see
            // `builtin_reflection_guarded_method`). String-literal and first-class-callable
            // constructions leave this `false` (fully backed, same as any named function).
            builtin_property("__unbacked_name", Visibility::Private, Some(TypeExpr::Bool), bool_lit(false)),
            // M2 PART A / JURY ADDENDUM item 2: SEPARATE from `__unbacked_name` — gates
            // `getFileName()`/`getStartLine()` independently of whether `getName()` is backed. A
            // closure-literal instance sets BOTH this and `__unbacked_name` `true` (unchanged
            // behavior). A dynamic descriptor-based instance (see
            // `crate::codegen_ir::lower_inst::objects::reflection_function_dynamic`) sets this
            // `true` while leaving `__unbacked_name` `false`, so `getName()` stays backed
            // (`"{closure}"` or the resolved real name) while `getFileName()`/`getStartLine()`
            // still throw — no per-value source-file/line tracking exists for ANY dynamically
            // reflected value. String-literal and first-class-callable static constructions leave
            // this `false` (unchanged: `getFileName()` reads the real `__file` slot).
            builtin_property("__unbacked_file", Visibility::Private, Some(TypeExpr::Bool), bool_lit(false)),
            // M2 PART A: gates `getParameters()` — SEPARATE from `__unbacked_name`/`__unbacked_file`
            // since `getNumberOfParameters()`/`getNumberOfRequiredParameters()` stay cheaply backed
            // for a dynamic descriptor (two register reads off its signature record — see
            // `reflection_function_dynamic`), but building the actual per-parameter
            // `ReflectionParameter[]` array would need a genuine RUNTIME loop over a
            // compile-time-unknown parameter count (the compile-time paths unroll this loop in
            // Rust — see `emit_reflection_parameter_array`); deferred rather than faked. Only the
            // dynamic construction path sets this `true`; the two static paths leave it `false`
            // (unchanged: `__params` stays the compile-time-baked array).
            builtin_property("__unbacked_params", Visibility::Private, Some(TypeExpr::Bool), bool_lit(false)),
            // M2 PART A: backs `isAnonymous()`. `true` for a closure LITERAL (always anonymous in
            // real PHP — php -n verified) and `false` for a named/first-class-callable static
            // construction; for a dynamic descriptor, computed at runtime from the descriptor's own
            // `kind` field (`CALLABLE_DESC_KIND_CLOSURE` → `true`, any other shape → `false` — a
            // wrapped/FCC-resolved target always has a real name in real PHP).
            builtin_property("__is_anonymous", Visibility::Private, Some(TypeExpr::Bool), bool_lit(false)),
            // N1 item 3: backs `getReturnType()`. `__return_type` holds the baked
            // `ReflectionNamedType` (boxed `Mixed`, `null` when the reflected target has no
            // declared return type — the SAME "backed but null" convention `ReflectionParameter`'s
            // `__type`/`getType()` already uses for an untyped parameter) for the two STATIC
            // construction paths (string-literal/first-class-callable named function, closure
            // literal — see `reflection_named_type_info`/`reflection_function_construction_metadata`
            // in `crate::codegen_ir::lower_inst::objects::reflection`). `__unbacked_return_type`
            // gates `getReturnType()` for the DYNAMIC descriptor-based construction path (see
            // `crate::codegen_ir::lower_inst::objects::reflection_function_dynamic`), which has no
            // per-value declared-return-type record to read at runtime — SEPARATE flag from
            // `__unbacked_params`/`__unbacked_file` (JURY ADDENDUM pattern), since a closure
            // LITERAL'S own return-type annotation IS statically known (unlike its declaring
            // file/line) and so stays backed even though `__unbacked_file`/`__unbacked_params` are
            // both `true` for that same instance.
            builtin_property("__return_type", Visibility::Private, Some(mixed_type()), null_lit()),
            builtin_property(
                "__unbacked_return_type",
                Visibility::Private,
                Some(TypeExpr::Bool),
                bool_lit(false),
            ),
        ],
        methods: vec![
            builtin_reflection_function_constructor_method(),
            builtin_reflection_function_get_file_name_method(),
            builtin_reflection_guarded_method(
                "getName",
                TypeExpr::Str,
                "__unbacked_name",
                "ReflectionFunction::getName() is not supported for a closure-literal-backed instance: elephc cannot reproduce PHP's per-closure name format (which embeds the declaring file/function and line)",
                this_prop_expr("__name"),
            ),
            builtin_reflection_guarded_method(
                "getShortName",
                TypeExpr::Str,
                "__unbacked_name",
                "ReflectionFunction::getShortName() is not supported for a closure-literal-backed instance: elephc cannot reproduce PHP's per-closure name format (which embeds the declaring file/function and line)",
                this_prop_expr("__short"),
            ),
            // Always throws: elephc tracks no declaration line number for ANY reflected
            // function/closure/method, static or dynamic (M2 PART A / JURY ADDENDUM item 2).
            builtin_reflection_unconditional_throw_method(
                "getStartLine",
                TypeExpr::Union(vec![TypeExpr::Int, TypeExpr::Bool]),
                "ReflectionFunction::getStartLine() is not supported: elephc does not track function/closure declaration line numbers",
                Vec::new(),
                None,
            ),
            // N1 item 3: same treatment as `getStartLine` above — elephc tracks no declaration
            // line numbers at all (start OR end), for ANY reflected function/closure/method,
            // static or dynamic. php -n VERIFIED signature: `getEndLine(): int|false`.
            builtin_reflection_unconditional_throw_method(
                "getEndLine",
                TypeExpr::Union(vec![TypeExpr::Int, TypeExpr::Bool]),
                "ReflectionFunction::getEndLine() is not supported: elephc does not track function/closure declaration line numbers",
                Vec::new(),
                None,
            ),
            builtin_reflection_slot_getter("getNumberOfParameters", "__num_params", TypeExpr::Int),
            builtin_reflection_slot_getter(
                "getNumberOfRequiredParameters",
                "__num_required",
                TypeExpr::Int,
            ),
            builtin_reflection_guarded_method(
                "getParameters",
                array_type(),
                "__unbacked_params",
                "ReflectionFunction::getParameters() is not supported for a dynamically-reflected instance: elephc does not yet build a runtime ReflectionParameter[] array for a compile-time-unknown parameter count",
                this_prop_expr("__params"),
            ),
            // N1 item 3: `getReturnType(): ?ReflectionType` (php -n VERIFIED). Backed for the two
            // STATIC construction paths (string-literal/first-class-callable named function,
            // closure literal) from the reflected target's OWN declared/inferred return type,
            // baked into `__return_type` at construction time — reusing the SAME
            // `reflection_named_type_info` machinery `ReflectionParameter::getType()` already
            // uses for parameters (see `crate::codegen_ir::lower_inst::objects::reflection`).
            // `__return_type` stays `null` (its declared default) when the target has no return
            // type annotation, matching `getType()`'s "backed but null" precedent for an untyped
            // parameter — NOT a throw, since "no declared return type" is itself a fully
            // determinable, real answer. Gated on `__unbacked_return_type` for a dynamically-
            // reflected instance, which has no per-value declared-return-type record to read.
            builtin_reflection_guarded_method(
                "getReturnType",
                mixed_type(),
                "__unbacked_return_type",
                "ReflectionFunction::getReturnType() is not supported for a dynamically-reflected instance: elephc does not track a per-value declared return type for a runtime callable descriptor",
                this_prop_expr("__return_type"),
            ),
            builtin_reflection_slot_getter("isAnonymous", "__is_anonymous", TypeExpr::Bool),
            // PHP: getClosureThis(): ?object — the bound `$this` of a closure, or null.
            // No runtime backing yet; returns null, typed `mixed` (covers `?object`).
            builtin_reflection_literal_method("getClosureThis", mixed_type(), null_lit()),
            // N1 item 3: `getClosureScopeClass(): ?ReflectionClass` (php -n VERIFIED: returns the
            // `ReflectionClass` of the class a closure was LEXICALLY DECLARED inside — i.e. the
            // class whose method body contains the closure literal, not necessarily the bound
            // `$this` object's class — or `null` for a closure with no declaring class, a plain
            // named function, or an FCC target). elephc's EIR module (`crate::ir::Function`) has
            // no "this closure's own lexical declaring-class scope" field at all (verified: no
            // such tracking exists anywhere in `src/ir/`, `src/ir_lower/`, or `src/codegen_ir/` —
            // only method/property DECLARING-class maps for member lookups, which answer a
            // different question). Always throws rather than faking a value or narrowing to the
            // bound-`$this` class, which would silently diverge from PHP for a static closure or
            // one bound to an unrelated object via `Closure::bindTo`.
            builtin_reflection_unconditional_throw_method(
                "getClosureScopeClass",
                mixed_type(),
                "ReflectionFunction::getClosureScopeClass() is not supported: elephc does not track a closure's lexical declaring-class scope",
                Vec::new(),
                None,
            ),
            // PHP real signature: `invoke(mixed ...$args): mixed` (variadic, php -n verified via
            // `ReflectionMethod("ReflectionFunction", "invoke")->getParameters()`). Always throws
            // (M2 PART A / JURY ADDENDUM item 5): no wiring yet through the SAME uniform closure
            // invoker `$closure(...)` direct calls use — the earlier `(?array $args=null)`-shaped
            // stub silently returning `null` on every call (any ctor path, not just dynamic) was
            // itself a "no stub" policy violation, fixed here to a loud guarded throw instead.
            builtin_reflection_unconditional_throw_method(
                "invoke",
                mixed_type(),
                "ReflectionFunction::invoke() is not supported: elephc does not yet dispatch through the uniform closure invoker from a Reflection object",
                Vec::new(),
                Some(("args", mixed_type())),
            ),
            // PHP real signature: `invokeArgs(array $args): mixed` (required array, no default —
            // php -n verified). Same guarded-throw rationale as `invoke()` above.
            builtin_reflection_unconditional_throw_method(
                "invokeArgs",
                mixed_type(),
                "ReflectionFunction::invokeArgs() is not supported: elephc does not yet dispatch through the uniform closure invoker from a Reflection object",
                vec![("args", Some(array_type()), None, false)],
                None,
            ),
            // PHP: getClosureCalledClass(): ?ReflectionClass. Un-backed stub
            // returning `null` typed `mixed` (object return → mixed).
            builtin_reflection_literal_method(
                "getClosureCalledClass",
                mixed_type(),
                null_lit(),
            ),
            builtin_reflection_unsupported_tostring_method("ReflectionFunction"),
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
        implements: vec!["Reflector".to_string()],
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
            // PHP: public readonly `string $name`. Recognition-level stub
            // (un-backed, default `''`) so `$param->name` type-checks; runtime
            // population is a separate follow-up.
            builtin_property("name", Visibility::Public, Some(TypeExpr::Str), empty_string()),
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
            builtin_reflection_unsupported_tostring_method("ReflectionParameter"),
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
        // `Reflector` (which extends `Stringable`) — see
        // `builtin_reflection_unsupported_tostring_method` for why the
        // `__toString()` contract it pulls in throws rather than stubbing.
        implements: vec!["Reflector".to_string()],
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
            // PHP: public readonly `string $name`. Recognition-level stub
            // (un-backed, default `''`) so `$rc->name` type-checks; runtime
            // population is a separate follow-up.
            builtin_property("name", Visibility::Public, Some(TypeExpr::Str), empty_string()),
            // -- construction-baked closed-world metadata slots (see
            //    `crate::codegen_ir::lower_inst::objects::reflection` for the
            //    EIR bakers) --
            builtin_property("__is_abstract", Visibility::Private, Some(TypeExpr::Bool), bool_lit(false)),
            // Declaring source file (empty-string sentinel for "unknown" — see
            // `builtin_reflection_get_file_name_method`), baked from
            // `crate::pipeline::scan_reflection_source_files`'s snapshot. Backs `getFileName()`.
            builtin_property("__file", Visibility::Private, Some(TypeExpr::Str), empty_string()),
            // Immediate parent class name (empty-string sentinel for "no parent"), baked at
            // construction time from `ClassInfo::parent`. Backs `getParentClass()`.
            builtin_property("__parent_name", Visibility::Private, Some(TypeExpr::Str), empty_string()),
            builtin_property("__is_final", Visibility::Private, Some(TypeExpr::Bool), bool_lit(false)),
            builtin_property("__is_interface", Visibility::Private, Some(TypeExpr::Bool), bool_lit(false)),
            builtin_property("__is_internal", Visibility::Private, Some(TypeExpr::Bool), bool_lit(false)),
            builtin_property("__short", Visibility::Private, Some(TypeExpr::Str), empty_string()),
            // Parent classes + all transitively implemented interfaces
            // (lowercased), excluding the reflected class itself. Backs
            // `isSubclassOf()`.
            builtin_property("__ancestors_lower", Visibility::Private, Some(str_array_type()), empty_array()),
            // All transitively implemented interfaces, exact case. Backs
            // `getInterfaceNames()`.
            builtin_property("__interfaces", Visibility::Private, Some(str_array_type()), empty_array()),
            // Same set, lowercased. Backs `implementsInterface()`.
            builtin_property("__interfaces_lower", Visibility::Private, Some(str_array_type()), empty_array()),
            // Own + inherited (non-private) method names, lowercased. Backs
            // `hasMethod()`.
            builtin_property("__methods_lower", Visibility::Private, Some(str_array_type()), empty_array()),
            // Own + inherited property names, exact case. Backs
            // `hasProperty()`.
            builtin_property("__properties", Visibility::Private, Some(str_array_type()), empty_array()),
            // K1 Part A: own + inherited method/property names, EXACT declared spelling, in
            // real PHP `getMethods()`/`getProperties()` declaration order (own class's own
            // declared order first, then each ancestor's own declared order appended) with
            // parent-private members already excluded (php -n verified — see
            // `crate::codegen::runtime::data::reflect_member_registry::
            // method_decl_order_and_names`/`property_decl_order_and_names`, which bake these).
            // Unlike `__methods_lower`/`__properties` above (unordered membership-test sets),
            // these back the actual `getMethods()`/`getProperties()` ENUMERATION bodies below.
            builtin_property("__methods_ordered", Visibility::Private, Some(str_array_type()), empty_array()),
            builtin_property("__properties_ordered", Visibility::Private, Some(str_array_type()), empty_array()),
            // Own + inherited class-constant names/values that fold to a
            // compile-time literal, in parallel index order. Back
            // `getConstants()`/`getConstant()`.
            builtin_property("__const_names", Visibility::Private, Some(str_array_type()), empty_array()),
            builtin_property("__const_values", Visibility::Private, Some(mixed_array_type()), empty_array()),
        ],
        methods: vec![
            // PHP: `__construct(object|string $objectOrClass)`. Modeled as `mixed` under
            // gradual typing so a literal `Str`, a variable of ANY type, or a real object all
            // type-check — the actual `object|string` boundary is enforced by the EIR dynamic
            // dispatcher at runtime (see `reflection_class_literal_arg` in
            // `crate::types::checker::inference::objects::constructors` and
            // `lower_reflection_class_new_dynamic` in
            // `crate::codegen_ir::lower_inst::objects::reflection`), matching PHP's own
            // runtime-only rejection of the wrong argument shape for this constructor.
            builtin_reflection_owner_constructor_method(vec![(
                "class_name",
                Some(mixed_type()),
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
            // PHP: `getMethod(string $name): ReflectionMethod`. Delegates to the dynamic
            // `ReflectionMethod($this->__name, $name)` constructor (J4: the flat method-table
            // dynamic-construction feature) — `$this->__name` is a RUNTIME string even for a
            // literally-constructed receiver, so this single body soundly serves both a
            // literal-constructed and a dynamically-constructed `ReflectionClass`. A miss throws
            // the SAME catchable `\ReflectionException` the constructor throws.
            builtin_reflection_string_arg_method(
                "getMethod",
                "name",
                TypeExpr::Named(Name::unqualified("ReflectionMethod")),
                Expr::new(
                    ExprKind::NewObject {
                        class_name: Name::unqualified("ReflectionMethod"),
                        args: vec![this_prop_expr("__name"), var_expr("name")],
                    },
                    crate::span::Span::dummy(),
                ),
            ),
            // PHP: `getProperty(string $name): ReflectionProperty`. Delegates to the dynamic
            // `ReflectionProperty($this->__name, $name)` constructor. See `getMethod` above.
            builtin_reflection_string_arg_method(
                "getProperty",
                "name",
                TypeExpr::Named(Name::unqualified("ReflectionProperty")),
                Expr::new(
                    ExprKind::NewObject {
                        class_name: Name::unqualified("ReflectionProperty"),
                        args: vec![this_prop_expr("__name"), var_expr("name")],
                    },
                    crate::span::Span::dummy(),
                ),
            ),
            // -- real, construction-baked closed-world metadata accessors --
            builtin_reflection_slot_getter("isAbstract", "__is_abstract", TypeExpr::Bool),
            builtin_reflection_slot_getter("isFinal", "__is_final", TypeExpr::Bool),
            builtin_reflection_slot_getter("isInterface", "__is_interface", TypeExpr::Bool),
            builtin_reflection_slot_getter("isInternal", "__is_internal", TypeExpr::Bool),
            // elephc's `ReflectionClass` constructor only ever resolves to a
            // real class (never a trait — traits are flattened into their
            // users and are not independently reflectable), so this is
            // soundly `false` in every reachable case, not a fabricated guess.
            builtin_reflection_literal_method("isTrait", TypeExpr::Bool, bool_lit(false)),
            builtin_reflection_computed_method(
                "isInstantiable",
                TypeExpr::Bool,
                Expr::new(
                    ExprKind::BinaryOp {
                        left: Box::new(Expr::new(
                            ExprKind::Not(Box::new(this_prop_expr("__is_abstract"))),
                            crate::span::Span::dummy(),
                        )),
                        op: BinOp::And,
                        right: Box::new(Expr::new(
                            ExprKind::Not(Box::new(this_prop_expr("__is_interface"))),
                            crate::span::Span::dummy(),
                        )),
                    },
                    crate::span::Span::dummy(),
                ),
            ),
            builtin_reflection_slot_getter("getShortName", "__short", TypeExpr::Str),
            builtin_reflection_slot_getter("getInterfaceNames", "__interfaces", str_array_type()),
            builtin_reflection_string_arg_method(
                "hasMethod",
                "name",
                TypeExpr::Bool,
                case_insensitive_method_membership_expr("name", "__methods_lower"),
            ),
            builtin_reflection_string_arg_method(
                "hasProperty",
                "name",
                TypeExpr::Bool,
                exact_membership_expr("name", "__properties"),
            ),
            // PHP: `getMethods(int $filter = 0): ReflectionMethod[]` / `getProperties(int
            // $filter = 0): ReflectionProperty[]` (K1 Part A). Both iterate the already
            // decl-order/private-filtered `__methods_ordered`/`__properties_ordered` slot,
            // constructing a shell per visible name through the EXISTING J4 dynamic
            // `ReflectionMethod($this->__name, $name)`/`ReflectionProperty(...)` dispatcher
            // (soundly serves both a literal- and a dynamically-constructed receiver, same as
            // `getMethod`/`getProperty` above), then keeps only entries matching `$filter`
            // (`0` = no filtering; otherwise PHP's OR-bitmask semantics `(modifiers & filter)
            // != 0`, php -n verified — see `get_members_body`).
            get_members_method("getMethods", "__methods_ordered", "ReflectionMethod"),
            get_members_method("getProperties", "__properties_ordered", "ReflectionProperty"),
            builtin_reflection_string_arg_method(
                "isSubclassOf",
                "class",
                TypeExpr::Bool,
                case_insensitive_membership_expr("class", "__ancestors_lower"),
            ),
            builtin_reflection_string_arg_method(
                "implementsInterface",
                "interface",
                TypeExpr::Bool,
                case_insensitive_membership_expr("interface", "__interfaces_lower"),
            ),
            {
                let dummy_span = crate::span::Span::dummy();
                ClassMethod {
                    name: "getConstants".to_string(),
                    visibility: Visibility::Public,
                    is_static: false,
                    is_abstract: false,
                    is_final: false,
                    has_body: true,
                    params: Vec::new(),
                    variadic: None,
                    variadic_type: None,
                    return_type: Some(mixed_array_type()),
                    by_ref_return: false,
                    body: get_constants_body("__const_names", "__const_values"),
                    span: dummy_span,
                    attributes: Vec::new(),
                }
            },
            {
                let dummy_span = crate::span::Span::dummy();
                ClassMethod {
                    name: "getConstant".to_string(),
                    visibility: Visibility::Public,
                    is_static: false,
                    is_abstract: false,
                    is_final: false,
                    has_body: true,
                    params: vec![("name".to_string(), Some(TypeExpr::Str), None, false)],
                    variadic: None,
                    variadic_type: None,
                    return_type: Some(mixed_type()),
                    by_ref_return: false,
                    body: get_constant_body("__const_names", "__const_values"),
                    span: dummy_span,
                    attributes: Vec::new(),
                }
            },
            builtin_reflection_unsupported_tostring_method("ReflectionClass"),
            builtin_reflection_get_file_name_method(),
            builtin_reflection_get_parent_class_method(),
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
    // PHP: public readonly `string $name` / `string $class`. Recognition-level
    // stubs (un-backed, default `''`) so `$rp->name` / `$rp->class` type-check;
    // runtime population is a separate follow-up.
    class
        .properties
        .push(builtin_property("name", Visibility::Public, Some(TypeExpr::Str), empty_string()));
    class
        .properties
        .push(builtin_property("class", Visibility::Public, Some(TypeExpr::Str), empty_string()));
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
    // Real PHP modifier bitmask (IS_STATIC|IS_PUBLIC|IS_PROTECTED|IS_PRIVATE),
    // baked at construction from the reflected property's actual visibility/
    // staticness — see `emit_reflection_property_modifiers` in the EIR codegen.
    class
        .properties
        .push(builtin_property("__modifiers", Visibility::Private, Some(TypeExpr::Int), int_lit(0)));
    // Whether the reflected property carries an explicit source type
    // declaration, baked at construction from `ClassInfo.declared_properties`/
    // `declared_static_properties` (the checker's own "has an explicit type
    // hint" bit — NOT the resolved `PhpType`, which an untyped property
    // still gets inferred, e.g. from its default value or `PhpType::Int` as
    // the no-default fallback; see `property_modifiers_and_type` in the EIR
    // codegen).
    class
        .properties
        .push(builtin_property("__has_declared_type", Visibility::Private, Some(TypeExpr::Bool), bool_lit(false)));
    class.implements.push("Reflector".to_string());
    class
        .methods
        .push(builtin_reflection_slot_getter("getModifiers", "__modifiers", TypeExpr::Int));
    class
        .methods
        .push(builtin_reflection_slot_getter("hasType", "__has_declared_type", TypeExpr::Bool));
    // Real visibility/staticness/readonly checks — single-bit tests against `__modifiers` (php -n
    // verified bit values: IS_PUBLIC=1, IS_PROTECTED=2, IS_PRIVATE=4, IS_STATIC=16,
    // IS_READONLY=128). Mirrors `ReflectionMethod`'s `isPublic`/`isProtected`/`isPrivate`/
    // `isStatic` accessors above.
    class.methods.push(builtin_reflection_computed_method(
        "isPublic",
        TypeExpr::Bool,
        modifier_bit_test_expr("__modifiers", 1),
    ));
    class.methods.push(builtin_reflection_computed_method(
        "isProtected",
        TypeExpr::Bool,
        modifier_bit_test_expr("__modifiers", 2),
    ));
    class.methods.push(builtin_reflection_computed_method(
        "isPrivate",
        TypeExpr::Bool,
        modifier_bit_test_expr("__modifiers", 4),
    ));
    class.methods.push(builtin_reflection_computed_method(
        "isStatic",
        TypeExpr::Bool,
        modifier_bit_test_expr("__modifiers", 16),
    ));
    class.methods.push(builtin_reflection_computed_method(
        "isReadOnly",
        TypeExpr::Bool,
        modifier_bit_test_expr("__modifiers", 128),
    ));
    class
        .methods
        .push(builtin_reflection_unsupported_tostring_method("ReflectionProperty"));
    // NOTE: `ReflectionProperty` deliberately has NO `getFileName()`/`__file` slot — php -n
    // verified real PHP does not declare that method on `ReflectionProperty` (only
    // `ReflectionClass` and `ReflectionFunctionAbstract`, i.e. `ReflectionFunction`/
    // `ReflectionMethod`, do): `(new ReflectionProperty(...))->getFileName()` is a hard
    // "Call to undefined method" fatal in real PHP. Adding it here would over-accept a method
    // PHP itself rejects.
    // PHP class constants — php -n verified (PHP 8.4+ added `IS_ABSTRACT`/
    // `IS_FINAL` for property hooks/final properties):
    // `ReflectionProperty::IS_STATIC=16, IS_PUBLIC=1, IS_PROTECTED=2,
    // IS_PRIVATE=4, IS_READONLY=128, IS_ABSTRACT=64, IS_FINAL=32`.
    for (name, value) in [
        ("IS_STATIC", 16),
        ("IS_PUBLIC", 1),
        ("IS_PROTECTED", 2),
        ("IS_PRIVATE", 4),
        ("IS_READONLY", 128),
        ("IS_ABSTRACT", 64),
        ("IS_FINAL", 32),
    ] {
        class.constants.push(ClassConst {
            name: name.to_string(),
            visibility: Visibility::Public,
            is_final: false,
            value: Expr::new(ExprKind::IntLiteral(value), crate::span::Span::dummy()),
            span: crate::span::Span::dummy(),
            attributes: Vec::new(),
        });
    }
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

/// Returns a public `getAttributes(?string $name = null, int $flags = 0)` method
/// that returns the private `__attrs` property as an `array` of
/// `ReflectionAttribute` objects. Filtering by name/flags is a runtime concern;
/// the stub returns all collected attributes regardless. The two optional params
/// exist so 1- and 2-arg calls type-check against PHP's signature.
fn builtin_reflection_owner_get_attributes_method() -> ClassMethod {
    let dummy_span = crate::span::Span::dummy();
    ClassMethod {
        name: "getAttributes".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![
            // PHP: getAttributes(?string $name = null, int $flags = 0). Filtering by name/flags
            // is a runtime concern; the stub returns all collected attributes regardless. The
            // params exist so 1- and 2-arg calls (e.g. getAttributes(AsCommand::class) and
            // getAttributes($class, ReflectionAttribute::IS_INSTANCEOF)) type-check against PHP.
            ("name".to_string(), Some(TypeExpr::Nullable(Box::new(TypeExpr::Str))), null_lit(), false),
            ("flags".to_string(), Some(TypeExpr::Int), int_lit(0), false),
        ],
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
