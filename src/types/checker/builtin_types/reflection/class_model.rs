//! Purpose:
//! Builds ReflectionClass, ReflectionObject, and ReflectionEnum shells.
//!
//! Called from:
//! - The Reflection checker metadata facade and sibling builders.
//!
//! Key details:
//! - Flattened inheritance exposes only methods supported by each synthetic owner.

use super::*;

/// Builds the `ReflectionClass` shell with retained eval metadata accessors.
pub(super) fn builtin_reflection_class() -> FlattenedClass {
    let mut class = builtin_reflection_class_shell();
    add_member_fill_preambles(&mut class, REFLECTION_CLASS_MEMBER_READERS);
    class
}

/// The `ReflectionClass` getters whose bodies read a member-reflector slot (`__methods`,
/// `__properties`, `__reflection_constants`, `__constructor`, `__parent_class`).
///
/// `test_every_member_slot_reader_fills_the_members_first` keeps this list complete.
const REFLECTION_CLASS_MEMBER_READERS: &[&str] = &[
    "getReflectionConstants",
    "getReflectionConstant",
    "getMethod",
    "getConstructor",
    "getParentClass",
    "getMethods",
    "getProperties",
    "getProperty",
    "isSubclassOf",
];

/// The `ReflectionEnum` getters whose bodies read its `__cases` member-reflector slot.
const REFLECTION_ENUM_MEMBER_READERS: &[&str] = &["getCases", "getCase", "hasCase"];

/// The member-reflector slots a compiled class reflector may leave for its filler.
#[cfg(test)]
const REFLECTION_MEMBER_SLOTS: &[&str] = &[
    "__interfaces",
    "__traits",
    "__reflection_constants",
    "__methods",
    "__constructor",
    "__parent_class",
    "__properties",
    "__cases",
];

/// Prepends `reflection_member_fill_preamble()` to each named method of `class`.
fn add_member_fill_preambles(class: &mut FlattenedClass, readers: &[&str]) {
    for method in &mut class.methods {
        if readers.contains(&method.name.as_str()) {
            method.body.insert(0, reflection_member_fill_preamble());
        }
    }
}

/// `if ($this->__members_filler !== 0) { __elephc_reflection_fill_members($this->__members_filler, $this); }`
///
/// A compiled class reflector records its members filler instead of building its member
/// reflectors (`emit_reflection_owner_object_with_members`); the filler clears the slot first,
/// so it runs once. A reflector the eval bridge built, or one filled up front, has 0 there.
fn reflection_member_fill_preamble() -> Stmt {
    let span = crate::span::Span::dummy();
    Stmt::new(
        StmtKind::If {
            condition: binary_expr(
                reflection_this_property("__members_filler", span),
                BinOp::StrictNotEq,
                Expr::new(ExprKind::IntLiteral(0), span),
                span,
            ),
            then_body: vec![Stmt::new(
                StmtKind::ExprStmt(function_call(
                    "__elephc_reflection_fill_members",
                    vec![
                        reflection_this_property("__members_filler", span),
                        Expr::new(ExprKind::This, span),
                    ],
                    span,
                )),
                span,
            )],
            elseif_clauses: Vec::new(),
            else_body: None,
        },
        span,
    )
}

/// Builds the `ReflectionClass` members and slots, before the member-fill preambles.
fn builtin_reflection_class_shell() -> FlattenedClass {
    FlattenedClass {
        name: "ReflectionClass".to_string(),
        span: dummy(),
        extends: None,
        implements: vec!["Reflector".to_string()],
        is_abstract: false,
        is_final: false,
        is_readonly_class: false,
        properties: vec![
            builtin_property(
                "__name",
                Visibility::Private,
                Some(TypeExpr::Str),
                empty_string(),
            ),
            builtin_property(
                "__doc_comment",
                Visibility::Private,
                Some(TypeExpr::Str),
                empty_string(),
            ),
            builtin_property(
                "__has_doc_comment",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__string",
                Visibility::Private,
                Some(TypeExpr::Str),
                empty_string(),
            ),
            builtin_property(
                "__attrs",
                Visibility::Private,
                Some(array_type()),
                empty_array(),
            ),
            builtin_property(
                "__is_final",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__is_abstract",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__is_interface",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__is_trait",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__is_enum",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__is_readonly",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__is_anonymous",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__is_instantiable",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__is_cloneable",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__is_iterable",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__is_internal",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__is_user_defined",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__modifiers",
                Visibility::Private,
                Some(TypeExpr::Int),
                int_lit(0),
            ),
            builtin_property(
                "__short_name",
                Visibility::Private,
                Some(TypeExpr::Str),
                empty_string(),
            ),
            builtin_property(
                "__namespace_name",
                Visibility::Private,
                Some(TypeExpr::Str),
                empty_string(),
            ),
            builtin_property(
                "__in_namespace",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            builtin_property(
                "__interface_names",
                Visibility::Private,
                Some(string_array_type()),
                empty_array(),
            ),
            builtin_property(
                "__interfaces",
                Visibility::Private,
                Some(object_array_type("ReflectionClass")),
                empty_array(),
            ),
            builtin_property(
                "__trait_names",
                Visibility::Private,
                Some(string_array_type()),
                empty_array(),
            ),
            builtin_property(
                "__traits",
                Visibility::Private,
                Some(object_array_type("ReflectionClass")),
                empty_array(),
            ),
            builtin_property(
                "__trait_aliases",
                Visibility::Private,
                Some(string_array_type()),
                empty_array(),
            ),
            builtin_property(
                "__parent_names",
                Visibility::Private,
                Some(string_array_type()),
                empty_array(),
            ),
            builtin_property(
                "__method_names",
                Visibility::Private,
                Some(string_array_type()),
                empty_array(),
            ),
            // Set only by the full compiled materializer: every slot the native bodies read is
            // filled, so calls on this reflector need not go through the eval bridge.
            builtin_property(
                "__complete",
                Visibility::Private,
                Some(bool_type()),
                false_bool(),
            ),
            // The address of the compiled filler that builds this reflector's member
            // reflectors, until a getter that reads them runs it; 0 once filled or when the
            // slots were filled up front (see `reflection_member_fill_preamble`).
            builtin_property(
                "__members_filler",
                Visibility::Private,
                Some(TypeExpr::Int),
                int_lit(0),
            ),
            builtin_property(
                "__property_names",
                Visibility::Private,
                Some(string_array_type()),
                empty_array(),
            ),
            builtin_property(
                "__constant_names",
                Visibility::Private,
                Some(string_array_type()),
                empty_array(),
            ),
            builtin_property(
                "__constants",
                Visibility::Private,
                Some(mixed_type()),
                empty_array(),
            ),
            builtin_property(
                "__default_properties",
                Visibility::Private,
                Some(mixed_type()),
                empty_array(),
            ),
            builtin_property(
                "__static_properties",
                Visibility::Private,
                Some(mixed_type()),
                empty_array(),
            ),
            builtin_property(
                "__reflection_constants",
                Visibility::Private,
                Some(object_array_type("ReflectionClassConstant")),
                empty_array(),
            ),
            builtin_property(
                "__methods",
                Visibility::Private,
                Some(object_array_type("ReflectionMethod")),
                empty_array(),
            ),
            builtin_property(
                "__constructor",
                Visibility::Private,
                Some(nullable_object_type("ReflectionMethod")),
                null_expr(),
            ),
            builtin_property(
                "__parent_class",
                Visibility::Private,
                Some(mixed_type()),
                false_bool(),
            ),
            builtin_property(
                "__properties",
                Visibility::Private,
                Some(object_array_type("ReflectionProperty")),
                empty_array(),
            ),
            builtin_property(
                "__file_name",
                Visibility::Private,
                Some(TypeExpr::Str),
                empty_string(),
            ),
            builtin_property(
                "__start_line",
                Visibility::Private,
                Some(TypeExpr::Int),
                int_lit(0),
            ),
            builtin_property(
                "__end_line",
                Visibility::Private,
                Some(TypeExpr::Int),
                int_lit(0),
            ),
        ],
        methods: vec![
            builtin_reflection_owner_constructor_method(vec![(
                "class_name",
                Some(class_string_or_object_type()),
                None,
                false,
            )]),
            builtin_reflection_class_string_method("getName", "__name"),
            builtin_reflection_class_string_method("__toString", "__string"),
            builtin_reflection_slot_getter(
                "getFileName",
                "__file_name",
                string_or_bool_type(),
            ),
            builtin_reflection_slot_getter(
                "getStartLine",
                "__start_line",
                int_or_bool_type(),
            ),
            builtin_reflection_slot_getter(
                "getEndLine",
                "__end_line",
                int_or_bool_type(),
            ),
            builtin_reflection_class_doc_comment_method(),
            builtin_reflection_constant_false_union_method("getExtensionName"),
            builtin_reflection_constant_null_mixed_method("getExtension"),
            builtin_reflection_class_string_method("getShortName", "__short_name"),
            builtin_reflection_class_string_method("getNamespaceName", "__namespace_name"),
            builtin_reflection_class_bool_method("inNamespace", "__in_namespace"),
            builtin_reflection_class_array_method(
                "getInterfaceNames",
                "__interface_names",
                string_array_type(),
            ),
            builtin_reflection_class_objects_from_names_method(
                "getInterfaces",
                "__interface_names",
                object_array_type("ReflectionClass"),
            ),
            builtin_reflection_class_array_method(
                "getTraitNames",
                "__trait_names",
                string_array_type(),
            ),
            builtin_reflection_class_objects_from_names_method(
                "getTraits",
                "__trait_names",
                object_array_type("ReflectionClass"),
            ),
            builtin_reflection_class_array_method(
                "getTraitAliases",
                "__trait_aliases",
                string_array_type(),
            ),
            builtin_reflection_class_bool_method("isFinal", "__is_final"),
            builtin_reflection_class_bool_method("isAbstract", "__is_abstract"),
            builtin_reflection_class_bool_method("isInterface", "__is_interface"),
            builtin_reflection_class_bool_method("isTrait", "__is_trait"),
            builtin_reflection_class_bool_method("isEnum", "__is_enum"),
            builtin_reflection_class_bool_method("isReadOnly", "__is_readonly"),
            builtin_reflection_class_bool_method("isAnonymous", "__is_anonymous"),
            builtin_reflection_class_bool_method("isInstantiable", "__is_instantiable"),
            builtin_reflection_class_bool_method("isCloneable", "__is_cloneable"),
            builtin_reflection_class_bool_method("isIterable", "__is_iterable"),
            builtin_reflection_class_bool_method("isIterateable", "__is_iterable"),
            builtin_reflection_class_bool_method("isInternal", "__is_internal"),
            builtin_reflection_class_bool_method("isUserDefined", "__is_user_defined"),
            builtin_reflection_class_int_method("getModifiers", "__modifiers"),
            builtin_reflection_class_has_name_method("hasMethod", "__method_names", true),
            builtin_reflection_class_has_name_method("hasProperty", "__property_names", false),
            builtin_reflection_class_has_name_method("hasConstant", "__constant_names", false),
            builtin_reflection_class_get_constant_method(),
            builtin_reflection_class_mixed_method("getConstants", "__constants"),
            builtin_reflection_class_mixed_method("getDefaultProperties", "__default_properties"),
            builtin_reflection_class_mixed_method("getStaticProperties", "__static_properties"),
            builtin_reflection_class_get_static_property_value_method(),
            builtin_reflection_class_set_static_property_value_method(),
            builtin_reflection_class_filtered_array_method(
                "getReflectionConstants",
                "__reflection_constants",
                object_array_type("ReflectionClassConstant"),
            ),
            builtin_reflection_class_get_reflection_constant_method(),
            builtin_reflection_class_implements_interface_method(),
            builtin_reflection_class_is_subclass_of_method(),
            builtin_reflection_class_is_instance_method(),
            builtin_reflection_class_get_member_method(
                "getMethod",
                "__methods",
                "ReflectionMethod",
                true,
            ),
            builtin_reflection_class_nullable_object_method(
                "getConstructor",
                "__constructor",
                "ReflectionMethod",
            ),
            builtin_reflection_class_mixed_method("getParentClass", "__parent_class"),
            builtin_reflection_class_filtered_array_method(
                "getMethods",
                "__methods",
                object_array_type("ReflectionMethod"),
            ),
            builtin_reflection_class_filtered_array_method(
                "getProperties",
                "__properties",
                object_array_type("ReflectionProperty"),
            ),
            builtin_reflection_class_get_member_method(
                "getProperty",
                "__properties",
                "ReflectionProperty",
                false,
            ),
            builtin_reflection_class_new_instance_method(),
            builtin_reflection_class_new_instance_args_method(),
            builtin_reflection_class_new_instance_without_constructor_method(),
            builtin_reflection_class_new_lazy_ghost_method(),
            builtin_reflection_class_new_lazy_proxy_method(),
            builtin_reflection_class_is_uninitialized_lazy_object_method(),
            builtin_reflection_class_lazy_object_identity_method("initializeLazyObject"),
            builtin_reflection_class_lazy_object_identity_method("markLazyObjectAsInitialized"),
            builtin_reflection_owner_get_attributes_method(),
        ],
        attributes: Vec::new(),
        constants: reflection_class_constants(),
        used_traits: Vec::new(),
        trait_aliases: Vec::new(),
    }
}

/// Builds the synthetic `ReflectionObject` class with ReflectionClass metadata slots.
pub(super) fn builtin_reflection_object_class() -> FlattenedClass {
    let mut class = builtin_reflection_class();
    class.name = "ReflectionObject".to_string();
    class.extends = Some("ReflectionClass".to_string());
    class.properties.push(builtin_property(
        "__object",
        Visibility::Private,
        Some(mixed_type()),
        null_expr(),
    ));
    if let Some(constructor) = class
        .methods
        .iter_mut()
        .find(|method| method.name == "__construct")
    {
        *constructor = builtin_reflection_owner_constructor_method(vec![(
            "object",
            Some(object_type()),
            None,
            false,
        )]);
    }
    class
}

/// Builds the synthetic `ReflectionEnum` class with flattened ReflectionClass members.
///
/// php's `ReflectionEnum` extends `ReflectionClass`, so it keeps every inherited method: the
/// materializer fills the same member slots for an enum as for a class. An allow-list of the
/// name and flag getters used to stand here, and `getMethods()` / `getConstants()` /
/// `implementsInterface()` on an enum were compile errors.
pub(super) fn builtin_reflection_enum_class() -> FlattenedClass {
    let mut class = builtin_reflection_class();
    class.name = "ReflectionEnum".to_string();
    class.properties.extend([
        builtin_property(
            "__case_names",
            Visibility::Private,
            Some(string_array_type()),
            empty_array(),
        ),
        builtin_property(
            "__cases",
            Visibility::Private,
            Some(array_type()),
            empty_array(),
        ),
        builtin_property(
            "__is_backed",
            Visibility::Private,
            Some(bool_type()),
            false_bool(),
        ),
        builtin_property(
            "__backing_type",
            Visibility::Private,
            Some(mixed_type()),
            null_expr(),
        ),
    ]);
    class.methods.extend([
        builtin_reflection_class_bool_method("isBacked", "__is_backed"),
        builtin_reflection_class_mixed_method("getBackingType", "__backing_type"),
        builtin_reflection_class_array_method("getCases", "__cases", array_type()),
        builtin_reflection_enum_get_case_method(),
        builtin_reflection_enum_has_case_method(),
    ]);
    add_member_fill_preambles(&mut class, REFLECTION_ENUM_MEMBER_READERS);
    class
}

/// Builds `ReflectionEnum::getCase(string $name)`: the case reflector of that name, or php's
/// `ReflectionException` -- "Case E::B does not exist", or "E::C is not a case" when the name is
/// one of the enum's plain constants.
fn builtin_reflection_enum_get_case_method() -> ClassMethod {
    let span = crate::span::Span::dummy();
    let name = variable_expr("name", span);
    let member = variable_expr("member", span);
    let found = binary_expr(
        method_call_expr(member.clone(), "getName", Vec::new(), span),
        BinOp::StrictEq,
        name.clone(),
        span,
    );
    let qualified = concat_expr(
        concat_expr(reflection_this_property("__name", span), string_lit("::", span), span),
        name.clone(),
        span,
    );
    let is_constant = function_call(
        "in_array",
        vec![
            name.clone(),
            reflection_this_property("__constant_names", span),
            Expr::new(ExprKind::BoolLiteral(true), span),
        ],
        span,
    );
    let body = vec![
        Stmt::new(
            StmtKind::Foreach {
                array: reflection_this_property("__cases", span),
                key_var: None,
                value_var: "member".to_string(),
                value_by_ref: false,
                body: vec![Stmt::new(
                    StmtKind::If {
                        condition: found,
                        then_body: vec![Stmt::new(StmtKind::Return(Some(member)), span)],
                        elseif_clauses: Vec::new(),
                        else_body: None,
                    },
                    span,
                )],
            },
            span,
        ),
        Stmt::new(
            StmtKind::If {
                condition: is_constant,
                then_body: vec![throw_new_reflection_exception(
                    concat_expr(qualified.clone(), string_lit(" is not a case", span), span),
                    span,
                )],
                elseif_clauses: Vec::new(),
                else_body: None,
            },
            span,
        ),
        throw_new_reflection_exception(
            concat_expr(
                concat_expr(string_lit("Case ", span), qualified, span),
                string_lit(" does not exist", span),
                span,
            ),
            span,
        ),
    ];
    ClassMethod {
        name: "getCase".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![("name".to_string(), Some(TypeExpr::Str), None, false)],
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(mixed_type()),
        by_ref_return: false,
        body,
        span,
        attributes: Vec::new(),
    }
}

/// Builds `ReflectionEnum::hasCase(string $name)`: whether the enum declares a case of exactly
/// that name (case names are case-sensitive, like constants).
fn builtin_reflection_enum_has_case_method() -> ClassMethod {
    let span = crate::span::Span::dummy();
    let contains = function_call(
        "in_array",
        vec![
            variable_expr("name", span),
            reflection_this_property("__case_names", span),
            Expr::new(ExprKind::BoolLiteral(true), span),
        ],
        span,
    );
    ClassMethod {
        name: "hasCase".to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params: vec![("name".to_string(), Some(TypeExpr::Str), None, false)],
        param_attributes: Vec::new(),
        variadic: None,
        variadic_by_ref: false,
        variadic_type: None,
        return_type: Some(bool_type()),
        by_ref_return: false,
        body: vec![Stmt::new(StmtKind::Return(Some(contains)), span)],
        span,
        attributes: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies every class-reflector method that reads a member-reflector slot runs the member
    /// filler first. A compiled reflector leaves those slots empty until then, so a reader
    /// missing from `REFLECTION_CLASS_MEMBER_READERS` / `REFLECTION_ENUM_MEMBER_READERS` would
    /// silently answer "no methods" for a class that has some.
    #[test]
    fn test_every_member_slot_reader_fills_the_members_first() {
        for class in [
            builtin_reflection_class(),
            builtin_reflection_object_class(),
            builtin_reflection_enum_class(),
        ] {
            for method in &class.methods {
                let body = format!("{:?}", method.body);
                let reads_member_slot = REFLECTION_MEMBER_SLOTS
                    .iter()
                    .any(|slot| body.contains(&format!("property: \"{}\"", slot)));
                if !reads_member_slot {
                    continue;
                }
                let first = method.body.first().map(|stmt| format!("{:?}", stmt));
                assert!(
                    first.is_some_and(|stmt| stmt.contains("__elephc_reflection_fill_members")),
                    "{}::{} reads a member-reflector slot without filling the members first",
                    class.name,
                    method.name
                );
            }
        }
    }
}
