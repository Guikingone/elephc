//! Purpose:
//! Synthesises built-in DOM extension checker metadata (`DOMNode`, `DOMDocument`,
//! `DOMElement`, `DOMText`, `DOMNodeList`, `DOMNamedNodeMap`) so vendor code that references
//! the PHP `ext-dom` classes — e.g. symfony/console's `XmlDescriptor.php` and
//! symfony/config's `XmlUtils.php` — type-checks.
//!
//! Called from:
//! - `crate::types::checker::driver::init` (immediately after `inject_builtin_reflection`).
//!
//! Key details:
//! - TYPE-CHECK ONLY: method bodies are minimal stubs (literal or parameter-passthrough
//!   returns) with no backing DOM runtime. Codegen/runtime support for these classes is a
//!   later, separate cluster; do not add runtime semantics here.
//! - Only the DOM surface actually exercised by `XmlDescriptor.php` is registered: the
//!   `DOMDocument` constructor, `createElement`, `createTextNode`, `saveXML`, `importNode`,
//!   `getElementsByTagName`, `DOMNode::appendChild`, `DOMElement::setAttribute`,
//!   `DOMNodeList::item`, and the `formatOutput`/`ownerDocument`/`childNodes` properties,
//!   plus the surface symfony/config's `XmlUtils` reaches: `DOMDocument::loadXML`,
//!   `normalizeDocument`, `schemaValidateSource`, `$validateOnParse`, and the
//!   `nodeValue`/`prefix`/`localName`/`attributes` node properties.
//!   `documentElement` is touched by neither file and is intentionally omitted.
//! - The two predicates that report success (`loadXML`, `schemaValidateSource`) return FALSE
//!   rather than true. With no XML parser behind these shells, claiming success would hand the
//!   caller an empty document as though it were the parsed configuration; returning false routes
//!   it into its own error path instead. A shell may answer "I could not", never "I did".
//! - `DOMNodeList` (returned by `getElementsByTagName()` and exposed as `DOMNode::$childNodes`)
//!   is a class not explicitly named in the original surface list but required for
//!   `foreach ($node->childNodes as $child)` and `->getElementsByTagName(...)->item(0)` to
//!   type-check. It implements `Iterator`/`Traversable`, mirroring the `DatePeriod` builtin
//!   iterator shell, so `foreach` over it resolves without extra special-casing.

use std::collections::{HashMap, HashSet};

use crate::errors::CompileError;
use crate::names::{php_symbol_key, Name};
use crate::parser::ast::{
    ClassMethod, ClassProperty, Expr, ExprKind, PropertyHooks, Stmt, StmtKind, TypeExpr, Visibility,
};
use crate::types::traits::FlattenedClass;

/// Returns a dummy source span for synthetic AST nodes.
fn dummy() -> crate::span::Span {
    crate::span::Span::dummy()
}

/// Returns a `TypeExpr::Named` reference to the given built-in DOM (or scalar
/// pseudo-type) class name.
fn class_type(name: &str) -> TypeExpr {
    TypeExpr::Named(Name::unqualified(name))
}

/// Returns a nullable `TypeExpr::Named` reference to the given class name (`?ClassName`).
fn nullable_class_type(name: &str) -> TypeExpr {
    TypeExpr::Nullable(Box::new(class_type(name)))
}

/// Returns a `TypeExpr` for the unqualified name `mixed`, used for the `Iterator`
/// contract's `current()`/`key()` return types.
fn mixed_type() -> TypeExpr {
    TypeExpr::Named(Name::unqualified("mixed"))
}

/// Returns the `mixed` type used for DOM string parameters that PHP coerces from
/// scalar arguments — e.g. `DOMElement::setAttribute`, whose `string $value` accepts
/// ints/floats/bools that PHP casts to string. Modelled as `mixed` under the gradual
/// type system so scalar callers type-check without weakening the shells' other typed
/// members. Vendor code such as symfony/console passes ints to these string params.
fn coercing_scalar() -> TypeExpr {
    TypeExpr::Named(Name::unqualified("mixed"))
}

/// Returns a `$name` variable-reference expression.
fn var(name: &str) -> Expr {
    Expr::new(ExprKind::Variable(name.to_string()), dummy())
}

/// Returns a `StringLiteral` expression with the given value.
fn str_lit(value: &str) -> Expr {
    Expr::new(ExprKind::StringLiteral(value.to_string()), dummy())
}

/// Returns an `IntLiteral` expression with the given value.
fn int_lit(value: i64) -> Expr {
    Expr::new(ExprKind::IntLiteral(value), dummy())
}

/// Returns a `BoolLiteral` expression with the given value.
fn bool_lit(value: bool) -> Expr {
    Expr::new(ExprKind::BoolLiteral(value), dummy())
}

/// Returns a `Null` literal expression.
fn null_lit() -> Expr {
    Expr::new(ExprKind::Null, dummy())
}

/// Returns a `new <class_name>()` object-construction expression with no arguments,
/// relying on the target class's implicit default constructor.
fn new_obj(class_name: &str) -> Expr {
    Expr::new(
        ExprKind::NewObject {
            class_name: Name::unqualified(class_name),
            args: Vec::new(),
        },
        dummy(),
    )
}

/// Returns a `return <expr>;` statement.
fn ret(value: Expr) -> Stmt {
    Stmt::new(StmtKind::Return(Some(value)), dummy())
}

/// Builds a `(name, type, default, by_ref)` method-parameter tuple for a required or
/// defaulted, non-by-ref parameter typed `ty`.
fn param(
    name: &str,
    ty: TypeExpr,
    default: Option<Expr>,
) -> (String, Option<TypeExpr>, Option<Expr>, bool) {
    (name.to_string(), Some(ty), default, false)
}

/// Builds a public, non-static, non-abstract instance method with the given name,
/// parameters, return type, and body.
fn method(
    name: &str,
    params: Vec<(String, Option<TypeExpr>, Option<Expr>, bool)>,
    return_type: Option<TypeExpr>,
    body: Vec<Stmt>,
) -> ClassMethod {
    ClassMethod {
        name: name.to_string(),
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        is_final: false,
        has_body: true,
        params,
        variadic: None,
        variadic_type: None,
        return_type,
        by_ref_return: false,
        body,
        span: dummy(),
        attributes: Vec::new(),
        param_attributes: Vec::new(),
        variadic_by_ref: false,
    }
}

/// Builds a public, non-static instance property with the given name, type, and
/// optional default expression (`None` leaves it an uninitialized typed property,
/// matching ordinary PHP typed-property semantics).
fn property(name: &str, type_expr: TypeExpr, default: Option<Expr>) -> ClassProperty {
    ClassProperty {
        name: name.to_string(),
        visibility: Visibility::Public,
        set_visibility: None,
        type_expr: Some(type_expr),
        hooks: PropertyHooks::none(),
        readonly: false,
        is_final: false,
        is_static: false,
        is_abstract: false,
        by_ref: false,
        default,
        span: dummy(),
        attributes: Vec::new(),
        is_promoted: false,
    }
}

/// Builds the `DOMNode` shell: the base class for every DOM node type. Nothing
/// instantiates `DOMNode` directly (`DOMDocument`/`DOMElement`/`DOMText` extend it),
/// so `is_abstract: false` is fine — it just never appears in a `new` expression in
/// the registered surface.
///
/// `appendChild()` is a stub that returns its own argument (already typed `DOMNode`),
/// which satisfies the declared return type without any backing tree structure.
/// `childNodes` has no default: it is an ordinary uninitialized typed property, since
/// there is no real DOM tree to seed it from yet.
fn builtin_dom_node() -> FlattenedClass {
    FlattenedClass {
        name: "DOMNode".to_string(),
        extends: None,
        implements: Vec::new(),
        is_abstract: false,
        is_final: false,
        is_readonly_class: false,
        properties: vec![
            property("ownerDocument", nullable_class_type("DOMDocument"), Some(null_lit())),
            property("childNodes", class_type("DOMNodeList"), None),
            // `nodeValue`, `prefix` and `attributes` live on `DOMNode` in PHP, not on the
            // subclasses, so registering them here covers `DOMElement`, `DOMText` and
            // `DOMDocument` uniformly — which is exactly how symfony/config's `XmlUtils`
            // reaches them (`$element->prefix`, `$element->attributes`, and `$node->nodeValue`
            // on a `$node instanceof \DOMText` narrowing).
            property("nodeValue", TypeExpr::Nullable(Box::new(TypeExpr::Str)), Some(null_lit())),
            property("prefix", TypeExpr::Str, Some(str_lit(""))),
            property("localName", TypeExpr::Str, Some(str_lit(""))),
            property("attributes", nullable_class_type("DOMNamedNodeMap"), Some(null_lit())),
        ],
        methods: vec![method(
            "appendChild",
            vec![param("node", class_type("DOMNode"), None)],
            Some(class_type("DOMNode")),
            vec![ret(var("node"))],
        )],
        attributes: Vec::new(),
        constants: Vec::new(),
        used_traits: Vec::new(),
        span: crate::span::Span::dummy(),
        trait_aliases: Vec::new(),
    }
}

/// Builds the `DOMDocument` shell (extends `DOMNode`): the document root, exposing the
/// `(version, encoding)` constructor plus the factory/serialization methods used by
/// `XmlDescriptor.php` (`createElement`, `createTextNode`, `saveXML`, `importNode`,
/// `getElementsByTagName`) and the `formatOutput` property. `appendChild`/`ownerDocument`/
/// `childNodes` are inherited from `DOMNode`.
fn builtin_dom_document() -> FlattenedClass {
    FlattenedClass {
        name: "DOMDocument".to_string(),
        extends: Some("DOMNode".to_string()),
        implements: Vec::new(),
        is_abstract: false,
        is_final: false,
        is_readonly_class: false,
        properties: vec![
            property("formatOutput", TypeExpr::Bool, Some(bool_lit(false))),
            property("validateOnParse", TypeExpr::Bool, Some(bool_lit(false))),
        ],
        methods: vec![
            method(
                "__construct",
                vec![
                    param("version", TypeExpr::Str, Some(str_lit("1.0"))),
                    param("encoding", TypeExpr::Str, Some(str_lit(""))),
                ],
                None,
                Vec::new(),
            ),
            method(
                "createElement",
                // PHP coerces the element's text value; accept `mixed` for it (keeping the
                // string tag name strict). The default stays the empty string.
                vec![
                    param("name", TypeExpr::Str, None),
                    param("value", coercing_scalar(), Some(str_lit(""))),
                ],
                Some(class_type("DOMElement")),
                vec![ret(new_obj("DOMElement"))],
            ),
            method(
                "createTextNode",
                // PHP coerces the text-node content; accept `mixed` so scalar callers pass.
                vec![param("data", coercing_scalar(), None)],
                Some(class_type("DOMText")),
                vec![ret(new_obj("DOMText"))],
            ),
            method(
                "saveXML",
                vec![param("node", nullable_class_type("DOMNode"), Some(null_lit()))],
                Some(TypeExpr::Str),
                vec![ret(str_lit(""))],
            ),
            method(
                "importNode",
                vec![
                    param("node", class_type("DOMNode"), None),
                    param("deep", TypeExpr::Bool, Some(bool_lit(false))),
                ],
                Some(class_type("DOMNode")),
                vec![ret(var("node"))],
            ),
            method(
                "getElementsByTagName",
                vec![param("qualifiedName", TypeExpr::Str, None)],
                Some(class_type("DOMNodeList")),
                vec![ret(new_obj("DOMNodeList"))],
            ),
            // The parse/validate surface symfony/config's `XmlUtils::parse()` calls. Both
            // predicates return FALSE, which is the honest shell answer: there is no XML
            // parser behind these classes, so claiming success would hand `XmlUtils` an empty
            // document and let a caller act on silently-missing configuration. Returning false
            // routes it into its own `XmlParsingException`/`InvalidXmlException` path, which is
            // a diagnostic rather than a wrong answer. `normalizeDocument()` is a genuine no-op
            // on a tree that is already empty, so its stub body is faithful as written.
            method(
                "loadXML",
                vec![
                    param("source", TypeExpr::Str, None),
                    param("options", TypeExpr::Int, Some(int_lit(0))),
                ],
                Some(TypeExpr::Bool),
                vec![ret(bool_lit(false))],
            ),
            method("normalizeDocument", Vec::new(), Some(TypeExpr::Void), Vec::new()),
            method(
                "schemaValidateSource",
                vec![
                    param("source", TypeExpr::Str, None),
                    param("flags", TypeExpr::Int, Some(int_lit(0))),
                ],
                Some(TypeExpr::Bool),
                vec![ret(bool_lit(false))],
            ),
        ],
        attributes: Vec::new(),
        constants: Vec::new(),
        used_traits: Vec::new(),
        span: crate::span::Span::dummy(),
        trait_aliases: Vec::new(),
    }
}

/// Builds the `DOMElement` shell (extends `DOMNode`): adds `setAttribute()`.
/// `appendChild`/`ownerDocument`/`childNodes` are inherited from `DOMNode`.
fn builtin_dom_element() -> FlattenedClass {
    FlattenedClass {
        name: "DOMElement".to_string(),
        extends: Some("DOMNode".to_string()),
        implements: Vec::new(),
        is_abstract: false,
        is_final: false,
        is_readonly_class: false,
        properties: Vec::new(),
        methods: vec![method(
            "setAttribute",
            // PHP: setAttribute(string $qualifiedName, string $value) — both coerce scalar
            // arguments, and vendor code passes ints, so accept `mixed` for each.
            vec![
                param("name", coercing_scalar(), None),
                param("value", coercing_scalar(), None),
            ],
            Some(TypeExpr::Void),
            Vec::new(),
        )],
        attributes: Vec::new(),
        constants: Vec::new(),
        used_traits: Vec::new(),
        span: crate::span::Span::dummy(),
        trait_aliases: Vec::new(),
    }
}

/// Builds the `DOMText` shell (extends `DOMNode`): a text node with no members of its
/// own in the registered surface. `appendChild`/`ownerDocument`/`childNodes` are
/// inherited from `DOMNode`.
fn builtin_dom_text() -> FlattenedClass {
    FlattenedClass {
        name: "DOMText".to_string(),
        extends: Some("DOMNode".to_string()),
        implements: Vec::new(),
        is_abstract: false,
        is_final: false,
        is_readonly_class: false,
        properties: Vec::new(),
        methods: Vec::new(),
        attributes: Vec::new(),
        constants: Vec::new(),
        used_traits: Vec::new(),
        span: crate::span::Span::dummy(),
        trait_aliases: Vec::new(),
    }
}

/// Builds the `DOMNodeList` shell: the result type of `DOMDocument::getElementsByTagName()`
/// and the type of `DOMNode::$childNodes`. Implements `Iterator`/`Traversable` (mirroring the
/// `DatePeriod` builtin iterator shell) purely so `foreach ($node->childNodes as $child)`
/// type-checks; the stub iteration methods carry no real cursor state. `item()` is the only
/// member `XmlDescriptor.php` calls directly.
fn builtin_dom_node_list() -> FlattenedClass {
    FlattenedClass {
        name: "DOMNodeList".to_string(),
        extends: None,
        implements: vec!["Iterator".to_string(), "Traversable".to_string()],
        is_abstract: false,
        is_final: false,
        is_readonly_class: false,
        properties: Vec::new(),
        methods: vec![
            method(
                "item",
                vec![param("index", TypeExpr::Int, None)],
                Some(nullable_class_type("DOMNode")),
                vec![ret(null_lit())],
            ),
            method("current", Vec::new(), Some(mixed_type()), vec![ret(null_lit())]),
            method("key", Vec::new(), Some(mixed_type()), vec![ret(int_lit(0))]),
            method("next", Vec::new(), Some(TypeExpr::Void), Vec::new()),
            method("rewind", Vec::new(), Some(TypeExpr::Void), Vec::new()),
            method("valid", Vec::new(), Some(TypeExpr::Bool), vec![ret(bool_lit(false))]),
        ],
        attributes: Vec::new(),
        constants: Vec::new(),
        used_traits: Vec::new(),
        span: crate::span::Span::dummy(),
        trait_aliases: Vec::new(),
    }
}

/// Builds the `DOMNamedNodeMap` shell: the type of `DOMNode::$attributes`, iterated by
/// symfony/config's `XmlUtils::convertDomElementToArray()`
/// (`foreach ($element->attributes as $name => $node)`). It mirrors `DOMNodeList` — the same
/// `Iterator`/`Traversable` stub surface — because the only thing this shell has to support is
/// that foreach; PHP's real map is keyed by attribute name, which `key()` reflects by returning
/// a string rather than `DOMNodeList`'s integer index.
fn builtin_dom_named_node_map() -> FlattenedClass {
    FlattenedClass {
        name: "DOMNamedNodeMap".to_string(),
        extends: None,
        implements: vec!["Iterator".to_string(), "Traversable".to_string()],
        is_abstract: false,
        is_final: false,
        is_readonly_class: false,
        properties: Vec::new(),
        methods: vec![
            method(
                "getNamedItem",
                vec![param("qualifiedName", TypeExpr::Str, None)],
                Some(nullable_class_type("DOMNode")),
                vec![ret(null_lit())],
            ),
            method(
                "item",
                vec![param("index", TypeExpr::Int, None)],
                Some(nullable_class_type("DOMNode")),
                vec![ret(null_lit())],
            ),
            method("current", Vec::new(), Some(mixed_type()), vec![ret(null_lit())]),
            method("key", Vec::new(), Some(mixed_type()), vec![ret(str_lit(""))]),
            method("next", Vec::new(), Some(TypeExpr::Void), Vec::new()),
            method("rewind", Vec::new(), Some(TypeExpr::Void), Vec::new()),
            method("valid", Vec::new(), Some(TypeExpr::Bool), vec![ret(bool_lit(false))]),
        ],
        attributes: Vec::new(),
        constants: Vec::new(),
        used_traits: Vec::new(),
        span: crate::span::Span::dummy(),
        trait_aliases: Vec::new(),
    }
}

/// Injects the six built-in DOM shell types (`DOMNode`, `DOMDocument`, `DOMElement`,
/// `DOMText`, `DOMNodeList`, `DOMNamedNodeMap`) into `class_map` after verifying none are
/// already declared.
/// Each type is a type-check-only shell; there is no DOM runtime yet, so method bodies are
/// minimal stubs. Returns an error if any DOM name is already in use by a user-declared
/// class, interface, or trait.
pub(crate) fn inject_builtin_dom(
    interface_map: &HashMap<String, super::InterfaceDeclInfo>,
    class_map: &mut HashMap<String, FlattenedClass>,
    trait_names: &HashSet<String>,
) -> Result<(), CompileError> {
    for builtin_name in [
        "DOMNode",
        "DOMDocument",
        "DOMElement",
        "DOMText",
        "DOMNodeList",
        "DOMNamedNodeMap",
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
                &format!("Cannot redeclare built-in DOM type: {}", builtin_name),
            ));
        }
    }

    class_map.insert("DOMNode".to_string(), builtin_dom_node());
    class_map.insert("DOMDocument".to_string(), builtin_dom_document());
    class_map.insert("DOMElement".to_string(), builtin_dom_element());
    class_map.insert("DOMText".to_string(), builtin_dom_text());
    class_map.insert("DOMNodeList".to_string(), builtin_dom_node_list());
    class_map.insert(
        "DOMNamedNodeMap".to_string(),
        builtin_dom_named_node_map(),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{ExprKind, StmtKind};

    /// A DOM shell may answer "I could not", never "I did".
    ///
    /// `loadXML()` and `schemaValidateSource()` both report FAILURE because there is no XML
    /// parser behind these classes. Reporting success would hand `XmlUtils::parse()` an empty
    /// document as though it were the parsed configuration, and the caller would go on to
    /// build a container out of silently-missing input; returning false routes it into its own
    /// `XmlParsingException`/`InvalidXmlException` path, which is a diagnostic rather than a
    /// wrong answer. This test pins that choice so a later "make it green" edit has to argue
    /// with it instead of flipping it by accident.
    #[test]
    fn parse_predicates_report_failure_not_success() {
        let document = builtin_dom_document();
        for name in ["loadXML", "schemaValidateSource"] {
            let method = document
                .methods
                .iter()
                .find(|method| method.name == name)
                .unwrap_or_else(|| panic!("{name} is registered on the DOMDocument shell"));
            let Some(StmtKind::Return(Some(value))) = method.body.first().map(|stmt| &stmt.kind)
            else {
                panic!("{name} must return a value");
            };
            assert!(
                matches!(value.kind, ExprKind::BoolLiteral(false)),
                "{name} must return false: a shell with no parser must not claim success"
            );
        }
    }

    /// `nodeValue`, `prefix`, `localName` and `attributes` belong to `DOMNode` in PHP, not to
    /// the leaf classes, so every node type must inherit them from one place. Registering them
    /// per-subclass would let `DOMText::$nodeValue` work while `DOMElement::$nodeValue` did
    /// not — a split symfony/config's `XmlUtils` walks straight across.
    #[test]
    fn node_level_properties_live_on_dom_node() {
        let node = builtin_dom_node();
        for name in ["nodeValue", "prefix", "localName", "attributes"] {
            assert!(
                node.properties.iter().any(|property| property.name == name),
                "DOMNode must carry ${name} so every node subclass inherits it"
            );
        }
        assert!(builtin_dom_element().properties.is_empty());
        assert!(builtin_dom_text().properties.is_empty());
    }
}
