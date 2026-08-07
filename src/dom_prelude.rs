//! Purpose:
//! The `ext-dom` class surface (`DOMNode`, `DOMDocument`, `DOMElement`, `DOMText`,
//! `DOMNodeList`, `DOMNamedNodeMap`), declared in Rust through `crate::synthetic_class` so it
//! compiles through the normal pipeline and therefore has EIR method bodies.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject`,
//!   after include resolution and before name resolution.
//!
//! Key details:
//! - This surface was first a set of checker-only shells (`FlattenedClass` values injected
//!   straight into the checker's class map). Those type-checked but never reached
//!   lowering, so `new DOMDocument()` died at the backend with "constructor call to
//!   DOMDocument::__construct without an emitted EIR method body" — every DOM-using
//!   program was un-compilable no matter how clean its types were. It then became PHP source
//!   text so it would be collected, checked, lowered and emitted like any user class.
//!   It now builds the SAME declarations as AST nodes directly: the pipeline still sees
//!   ordinary class declarations, but the compiler no longer carries a PHP program inside a
//!   string literal. The MEMBER SET and the STUB BODIES are unchanged across all three forms,
//!   so each swap was a delivery-mechanism change, not a semantic one.
//! - Still incomplete on purpose: there is no XML PARSER behind these classes. `loadXML()`
//!   and `schemaValidateSource()` report FAILURE rather than success — a shell may answer
//!   "I could not", never "I did". Claiming success would hand `XmlUtils::parse()` an empty
//!   document as though it were the parsed configuration and let a container be built from
//!   silently-missing input; returning false routes it into its own `XmlParsingException`
//!   path instead.
//! - Injected UNCONDITIONALLY, matching the reach the checker shells had (they were
//!   registered for every program). Unused classes are dropped by the closed-world prune,
//!   so a non-DOM binary carries nothing extra.
//! - Unused PARAMETERS are consumed by a synthesized `$_unused = …;` statement rather than
//!   renamed — `uses()` names the parameters a body reads and `synthetic_class` consumes the
//!   rest. See that module for why the consumption is needed and why renaming is not an option.

use crate::parser::ast::{Expr, Program, TypeExpr};
use crate::synthetic_class::{class, e_bool, e_new, e_null, method, t_class, t_mixed, t_nullable};

/// Builds the DOM class surface.
///
/// `DOMNodeList` and `DOMNamedNodeMap` implement `Iterator` so `foreach ($node->childNodes
/// as $child)` and `foreach ($element->attributes as $name => $attr)` resolve; their cursor
/// methods carry no real state, mirroring the shells they replace. `DOMNamedNodeMap::key()`
/// returns a string rather than an index, matching PHP's name-keyed map.
fn dom_classes() -> Program {
    vec![
        class("DOMNodeList")
            .implements("Iterator")
            .method(
                method("item")
                    .param("index", TypeExpr::Int)
                    .returns(t_nullable(t_class("DOMNode")))
                    .returning(e_null()),
            )
            .method(method("current").returns(t_mixed()).returning(e_null()))
            .method(
                method("key")
                    .returns(t_mixed())
                    .returning(Expr::int_lit(0)),
            )
            .method(method("next").returns(TypeExpr::Void))
            .method(method("rewind").returns(TypeExpr::Void))
            .method(
                method("valid")
                    .returns(TypeExpr::Bool)
                    .returning(e_bool(false)),
            )
            .build(),
        class("DOMNamedNodeMap")
            .implements("Iterator")
            .method(
                method("getNamedItem")
                    .param("qualifiedName", TypeExpr::Str)
                    .returns(t_nullable(t_class("DOMNode")))
                    .returning(e_null()),
            )
            .method(
                method("item")
                    .param("index", TypeExpr::Int)
                    .returns(t_nullable(t_class("DOMNode")))
                    .returning(e_null()),
            )
            .method(method("current").returns(t_mixed()).returning(e_null()))
            .method(
                method("key")
                    .returns(t_mixed())
                    .returning(Expr::string_lit("")),
            )
            .method(method("next").returns(TypeExpr::Void))
            .method(method("rewind").returns(TypeExpr::Void))
            .method(
                method("valid")
                    .returns(TypeExpr::Bool)
                    .returning(e_bool(false)),
            )
            .build(),
        class("DOMNode")
            .prop(
                "ownerDocument",
                t_nullable(t_class("DOMDocument")),
                Some(e_null()),
            )
            // Declared without a default, exactly as the PHP form was: a typed property with
            // no initializer, left for a real implementation to populate.
            .prop("childNodes", t_class("DOMNodeList"), None)
            .prop("nodeValue", t_nullable(TypeExpr::Str), Some(e_null()))
            .prop("prefix", TypeExpr::Str, Some(Expr::string_lit("")))
            .prop("localName", TypeExpr::Str, Some(Expr::string_lit("")))
            .prop(
                "attributes",
                t_nullable(t_class("DOMNamedNodeMap")),
                Some(e_null()),
            )
            .method(
                method("appendChild")
                    .param("node", t_class("DOMNode"))
                    .returns(t_class("DOMNode"))
                    .uses(&["node"])
                    .returning(Expr::var("node")),
            )
            .build(),
        class("DOMDocument")
            .extends("DOMNode")
            .prop("formatOutput", TypeExpr::Bool, Some(e_bool(false)))
            .prop("validateOnParse", TypeExpr::Bool, Some(e_bool(false)))
            .method(
                method("__construct")
                    .param_default("version", TypeExpr::Str, Expr::string_lit("1.0"))
                    .param_default("encoding", TypeExpr::Str, Expr::string_lit("")),
            )
            .method(
                method("createElement")
                    .param("name", TypeExpr::Str)
                    .param_default("value", t_mixed(), Expr::string_lit(""))
                    .returns(t_class("DOMElement"))
                    .returning(e_new("DOMElement")),
            )
            .method(
                method("createTextNode")
                    .param("data", t_mixed())
                    .returns(t_class("DOMText"))
                    .returning(e_new("DOMText")),
            )
            .method(
                method("saveXML")
                    .param_default("node", t_nullable(t_class("DOMNode")), e_null())
                    .returns(TypeExpr::Str)
                    .returning(Expr::string_lit("")),
            )
            .method(
                method("importNode")
                    .param("node", t_class("DOMNode"))
                    .param_default("deep", TypeExpr::Bool, e_bool(false))
                    .returns(t_class("DOMNode"))
                    .uses(&["node"])
                    .returning(Expr::var("node")),
            )
            .method(
                method("getElementsByTagName")
                    .param("qualifiedName", TypeExpr::Str)
                    .returns(t_class("DOMNodeList"))
                    .returning(e_new("DOMNodeList")),
            )
            // Reports FAILURE, never success — see the module header.
            .method(
                method("loadXML")
                    .param("source", TypeExpr::Str)
                    .param_default("options", TypeExpr::Int, Expr::int_lit(0))
                    .returns(TypeExpr::Bool)
                    .returning(e_bool(false)),
            )
            .method(method("normalizeDocument").returns(TypeExpr::Void))
            .method(
                method("schemaValidateSource")
                    .param("source", TypeExpr::Str)
                    .param_default("flags", TypeExpr::Int, Expr::int_lit(0))
                    .returns(TypeExpr::Bool)
                    .returning(e_bool(false)),
            )
            .build(),
        class("DOMElement")
            .extends("DOMNode")
            .method(
                method("setAttribute")
                    .param("name", t_mixed())
                    .param("value", t_mixed())
                    .returns(TypeExpr::Void),
            )
            .build(),
        class("DOMText").extends("DOMNode").build(),
    ]
}

/// Prepends the DOM class surface to the program.
///
/// Injected unconditionally: the checker shells this replaces were registered for every
/// program, so gating injection on a usage scan would NARROW the surface rather than
/// preserve it. The prelude carries only declarations, which are discovered
/// position-independently, so prepending it does not change top-level execution order.
pub fn inject(program: Program) -> Program {
    let mut combined = dom_classes();
    combined.extend(program);
    combined
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::StmtKind;

    /// The surface is fixed: six classes in dependency-free declaration order. A member
    /// disappearing silently is exactly the failure the PHP form could not catch, since a
    /// typo there produced a parse error only when the text happened to be malformed.
    #[test]
    fn declares_the_six_dom_classes() {
        let names: Vec<String> = dom_classes()
            .iter()
            .filter_map(|stmt| match &stmt.kind {
                StmtKind::ClassDecl { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            names,
            vec![
                "DOMNodeList",
                "DOMNamedNodeMap",
                "DOMNode",
                "DOMDocument",
                "DOMElement",
                "DOMText",
            ]
        );
    }

    /// Every declared parameter must be read by its method body, either by the body itself
    /// or by the synthesized `$_unused` consumption. An unread parameter emits a warning on
    /// EVERY compile, including programs that never touch the DOM, because the injection is
    /// unconditional.
    #[test]
    fn every_parameter_is_consumed() {
        for stmt in dom_classes() {
            let StmtKind::ClassDecl { name, methods, .. } = &stmt.kind else {
                continue;
            };
            for m in methods {
                if m.params.is_empty() {
                    continue;
                }
                let mut referenced = Vec::new();
                collect_variables(&m.body, &mut referenced);
                for (param, _, _, _) in &m.params {
                    assert!(
                        referenced.iter().any(|v| v == param),
                        "{}::{} leaves ${} unread",
                        name,
                        m.name,
                        param
                    );
                }
            }
        }
    }

    /// Collects the variable names read anywhere in these bodies. The vocabulary is limited
    /// to what the shells actually contain: `return <expr>;` and the synthesized
    /// `$_unused = $p;` / `$_unused = [$a, $b];` assignment.
    fn collect_variables(body: &[crate::parser::ast::Stmt], out: &mut Vec<String>) {
        fn walk(expr: &Expr, out: &mut Vec<String>) {
            match &expr.kind {
                crate::parser::ast::ExprKind::Variable(name) => out.push(name.clone()),
                crate::parser::ast::ExprKind::ArrayLiteral(items) => {
                    for item in items {
                        walk(item, out);
                    }
                }
                crate::parser::ast::ExprKind::Assignment { target, value, .. } => {
                    walk(target, out);
                    walk(value, out);
                }
                _ => {}
            }
        }
        for stmt in body {
            match &stmt.kind {
                StmtKind::Return(Some(expr)) | StmtKind::ExprStmt(expr) => walk(expr, out),
                _ => {}
            }
        }
    }
}
