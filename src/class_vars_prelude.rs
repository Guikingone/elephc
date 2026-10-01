//! Purpose:
//! Builds PHP's `get_class_vars()` over the program's own classes as prelude declarations, for
//! programs that name it.
//!
//! Called from:
//! - `crate::backend_gap_prelude::inject_if_used`, which prepends these declarations when the
//!   program references `get_class_vars`, after autoload expansion has exposed every class.
//! - `crate::ir_lower::expr::compat_preludes::lower_get_class_vars`, which redirects each call
//!   here with the calling scope it knows lexically.
//!
//! Key details:
//! - The backend had no `get_class_vars` at all (an eval-only contract), so a compiled call was
//!   an uncatchable `Call to undefined function`; Eloquent's `Model` reads
//!   `get_class_vars($class)['timestamps']` on every model.
//! - One `switch` arm per class that declares properties, holding that class's DEFAULT values:
//!   the declared default expressions themselves, with `self::`/`parent::`/`__CLASS__` resolved
//!   to the declaring class, and `null` for a property without one (an uninitialized typed
//!   property included). php reports defaults for static properties too, never current values.
//! - MEASURED against php 8.5, and modeled exactly:
//!   * instance properties first, then static ones (php walks the table twice);
//!   * a class's own declarations (its traits' after them), then the inherited ones it does not
//!     redeclare, in the parent's order;
//!   * visibility follows the CALLING scope: a private property only from its declaring class, a
//!     protected one from any class related to it by inheritance in either direction;
//!   * an unknown name is php's `TypeError`.
//! - A class with no property declarations of its own needs no arm: the fallback walks
//!   `get_parent_class()` to the nearest one that has. A trait has an arm of its own, holding
//!   its declarations as the trait's.
//! - `get_class_vars` itself is declared too, for callable strings, which have no lexical scope.
//! - The declarations are BUILT, never parsed: elephc does not parse PHP source outside tests.

use std::collections::HashMap;

use crate::parser::ast::{
    BinOp, ClassProperty, Expr, ExprKind, MagicConstant, Program, StaticReceiver, Stmt, StmtKind,
    TraitUse, TypeExpr, Visibility,
};
use crate::synthetic_class::{
    class_name, e_array, e_binop, e_bool, e_call, e_index, e_new, e_not, e_null, e_str, e_var,
    function, internal_declarations, s_array_assign, s_assign, s_expr, s_if, s_return, s_switch,
    s_throw, s_typed_assign, t_array, t_mixed,
};

/// Entry point the lowering redirects `get_class_vars($class)` to, with the calling scope.
pub(crate) const GET_CLASS_VARS_NAME: &str = "__elephc_get_class_vars";

/// One class's visible defaults, instance or static, including what it inherits.
const CLASS_VARS_OF_NAME: &str = "__elephc_class_vars_of";

/// Returns the `get_class_vars()` declarations for `program`'s classes.
pub(crate) fn declarations(program: &Program) -> Program {
    let mut classes = Vec::new();
    let mut traits = HashMap::new();
    collect_class_likes(program, &mut classes, &mut traits);
    internal_declarations(|| {
        vec![
            decl_fn_get_class_vars_wrapper(),
            decl_fn_get_class_vars(),
            decl_fn_class_vars_of(&classes, &traits),
        ]
    })
}

/// One class declaration as `get_class_vars()` sees it.
struct ClassShape {
    name: String,
    parent: Option<String>,
    properties: Vec<ClassProperty>,
}

/// Collects every class and trait declaration, including ones nested in conditional blocks.
fn collect_class_likes(
    stmts: &[Stmt],
    classes: &mut Vec<ClassShape>,
    traits: &mut HashMap<String, (Vec<TraitUse>, Vec<ClassProperty>)>,
) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::ClassDecl {
                name,
                extends,
                trait_uses,
                properties,
                ..
            } => classes.push(ClassShape {
                name: name.trim_start_matches('\\').to_string(),
                parent: extends
                    .as_ref()
                    .map(|parent| parent.as_str().trim_start_matches('\\').to_string()),
                // Trait properties are resolved once every trait has been collected.
                properties: properties
                    .iter()
                    .cloned()
                    .chain(std::iter::once(trait_marker(trait_uses)))
                    .collect(),
            }),
            StmtKind::TraitDecl {
                name,
                trait_uses,
                properties,
                ..
            } => {
                traits.insert(
                    symbol_key(name),
                    (trait_uses.clone(), properties.clone()),
                );
                classes.push(ClassShape {
                    name: name.trim_start_matches('\\').to_string(),
                    parent: None,
                    properties: properties
                        .iter()
                        .cloned()
                        .chain(std::iter::once(trait_marker(trait_uses)))
                        .collect(),
                });
            }
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                collect_class_likes(then_body, classes, traits);
                for (_, body) in elseif_clauses {
                    collect_class_likes(body, classes, traits);
                }
                if let Some(body) = else_body {
                    collect_class_likes(body, classes, traits);
                }
            }
            StmtKind::NamespaceBlock { body, .. } | StmtKind::Synthetic(body) => {
                collect_class_likes(body, classes, traits);
            }
            StmtKind::Try {
                try_body,
                finally_body,
                ..
            } => {
                collect_class_likes(try_body, classes, traits);
                if let Some(body) = finally_body {
                    collect_class_likes(body, classes, traits);
                }
            }
            _ => {}
        }
    }
}

/// Name of the placeholder property that carries a class's trait uses to the arm builder.
const TRAIT_MARKER: &str = "\0traits";

/// Wraps a class's trait uses in a placeholder property, so `ClassShape` stays one list in
/// declaration order until the traits are known.
fn trait_marker(trait_uses: &[TraitUse]) -> ClassProperty {
    let names = trait_uses
        .iter()
        .flat_map(|trait_use| trait_use.trait_names.iter())
        .map(|name| name.as_str().trim_start_matches('\\').to_string())
        .collect::<Vec<_>>()
        .join("\0");
    ClassProperty {
        name: TRAIT_MARKER.to_string(),
        visibility: Visibility::Public,
        set_visibility: None,
        type_expr: None,
        hooks: Default::default(),
        readonly: false,
        is_final: false,
        is_static: false,
        is_abstract: false,
        by_ref: false,
        is_promoted: false,
        default: Some(e_str(&names)),
        span: crate::span::Span::dummy(),
        attributes: Vec::new(),
    }
}

/// php's case-insensitive class key.
fn symbol_key(name: &str) -> String {
    name.trim_start_matches('\\').to_ascii_lowercase()
}

/// Resolves a class's property list: its own declarations, then its traits' (recursively),
/// without a name a closer declaration already supplied.
fn resolved_properties(
    class: &ClassShape,
    traits: &HashMap<String, (Vec<TraitUse>, Vec<ClassProperty>)>,
) -> Vec<ClassProperty> {
    let mut properties: Vec<ClassProperty> = Vec::new();
    let mut trait_names = Vec::new();
    for property in &class.properties {
        if property.name == TRAIT_MARKER {
            if let Some(Expr { kind: ExprKind::StringLiteral(names), .. }) = &property.default {
                trait_names.extend(names.split('\0').filter(|n| !n.is_empty()).map(str::to_string));
            }
        } else {
            properties.push(property.clone());
        }
    }
    let mut seen = Vec::new();
    for trait_name in trait_names {
        append_trait_properties(&trait_name, traits, &mut properties, &mut seen);
    }
    properties
}

/// Appends one trait's properties, then the properties of the traits it uses.
fn append_trait_properties(
    trait_name: &str,
    traits: &HashMap<String, (Vec<TraitUse>, Vec<ClassProperty>)>,
    properties: &mut Vec<ClassProperty>,
    seen: &mut Vec<String>,
) {
    let key = symbol_key(trait_name);
    if seen.contains(&key) {
        return;
    }
    seen.push(key.clone());
    let Some((uses, own)) = traits.get(&key) else {
        return;
    };
    for property in own {
        if !properties.iter().any(|existing| existing.name == property.name) {
            properties.push(property.clone());
        }
    }
    for trait_use in uses {
        for nested in &trait_use.trait_names {
            append_trait_properties(nested.as_str(), traits, properties, seen);
        }
    }
}

/// `function get_class_vars(mixed $class): mixed` — the declared name, for callable uses.
///
/// A direct call never reaches it: the lowering redirects `get_class_vars($c)` to
/// [`GET_CLASS_VARS_NAME`] with the lexical scope. A callable string (`call_user_func(
/// 'get_class_vars', …)`) has no lexical class to pass, so it answers for the global scope.
fn decl_fn_get_class_vars_wrapper() -> Stmt {
    function("get_class_vars")
        .param("class", t_mixed())
        .returns(t_mixed())
        .body(vec![s_return(e_call(
            GET_CLASS_VARS_NAME,
            vec![e_var("class"), e_str("")],
        ))])
        .build()
}

/// `function __elephc_get_class_vars(mixed $class, string $scope): mixed`.
///
/// Validates the name the way php does, then joins the instance and the static defaults.
fn decl_fn_get_class_vars() -> Stmt {
    let key = e_call(
        "strtolower",
        vec![e_call("ltrim", vec![e_var("class"), e_str("\\")])],
    );
    let exists = ["class_exists", "interface_exists", "trait_exists", "enum_exists"]
        .into_iter()
        .map(|probe| e_call(probe, vec![e_var("class")]))
        .reduce(|left, right| e_binop(left, BinOp::Or, right))
        .expect("four existence probes");
    let message = e_binop(
        e_binop(
            e_str("get_class_vars(): Argument #1 ($class) must be a valid class name, "),
            BinOp::Concat,
            e_var("class"),
        ),
        BinOp::Concat,
        e_str(" given"),
    );
    function(GET_CLASS_VARS_NAME)
        .param("class", t_mixed())
        .param("scope", TypeExpr::Str)
        .returns(t_mixed())
        .body(vec![
            s_if(
                e_not(exists),
                vec![s_throw(e_new("TypeError", vec![message]))],
                Vec::new(),
                None,
            ),
            s_assign("key", key),
            s_return(e_binop(
                e_call(
                    CLASS_VARS_OF_NAME,
                    vec![e_var("key"), e_var("scope"), e_bool(false)],
                ),
                BinOp::Add,
                e_call(
                    CLASS_VARS_OF_NAME,
                    vec![e_var("key"), e_var("scope"), e_bool(true)],
                ),
            )),
        ])
        .build()
}

/// `function __elephc_class_vars_of(string $class, string $scope, bool $statics): array`.
fn decl_fn_class_vars_of(
    classes: &[ClassShape],
    traits: &HashMap<String, (Vec<TraitUse>, Vec<ClassProperty>)>,
) -> Stmt {
    let mut arms = Vec::new();
    let mut seen = Vec::new();
    for class in classes {
        let key = symbol_key(&class.name);
        // A conditionally declared class can appear twice; php declares one of them, and both
        // spell the same defaults in practice.
        if seen.contains(&key) {
            continue;
        }
        let properties = resolved_properties(class, traits);
        if properties.is_empty() {
            continue;
        }
        seen.push(key.clone());
        arms.push((vec![e_str(&key)], class_arm(class, &properties)));
    }
    // A class without declarations of its own inherits its parent's answer unchanged.
    let fallback = vec![
        s_assign("parent", e_call("get_parent_class", vec![e_var("class")])),
        s_if(
            e_binop(
                e_binop(e_var("parent"), BinOp::StrictEq, e_bool(false)),
                BinOp::Or,
                e_binop(
                    e_call("strtolower", vec![e_var("parent")]),
                    BinOp::StrictEq,
                    e_var("class"),
                ),
            ),
            vec![s_return(e_array(Vec::new()))],
            Vec::new(),
            None,
        ),
        s_return(e_call(
            CLASS_VARS_OF_NAME,
            vec![
                e_call("strtolower", vec![e_var("parent")]),
                e_var("scope"),
                e_var("statics"),
            ],
        )),
    ];
    let mut body = Vec::new();
    if !arms.is_empty() {
        body.push(s_switch(e_var("class"), arms, None));
    }
    body.extend(fallback);
    function(CLASS_VARS_OF_NAME)
        .param("class", TypeExpr::Str)
        .param("scope", TypeExpr::Str)
        .param("statics", TypeExpr::Bool)
        .returns(t_array())
        .body(body)
        .build()
}

/// One class's arm: its own visible defaults, then what it inherits and does not redeclare.
fn class_arm(class: &ClassShape, properties: &[ClassProperty]) -> Vec<Stmt> {
    let assigns = |statics: bool| -> Vec<Stmt> {
        properties
            .iter()
            .filter(|property| property.is_static == statics && !property.is_abstract)
            .flat_map(|property| visible_assign(class, property))
            .collect()
    };
    let mut body = vec![
        s_assign("vars", e_array(Vec::new())),
        s_if(e_var("statics"), assigns(true), Vec::new(), Some(assigns(false))),
    ];
    let Some(parent) = &class.parent else {
        body.push(s_return(e_var("vars")));
        return body;
    };
    body.push(s_assign(
        "inherited",
        e_call(
            CLASS_VARS_OF_NAME,
            vec![e_str(&symbol_key(parent)), e_var("scope"), e_var("statics")],
        ),
    ));
    // A redeclaration replaces the inherited property even where the caller cannot see the new
    // one; a visible redeclaration is already first in `$vars`, which `+` keeps.
    for property in properties {
        if property.visibility != Visibility::Public {
            body.push(s_expr(e_call(
                "unset",
                vec![e_index(e_var("inherited"), e_str(&property.name))],
            )));
        }
    }
    body.push(s_return(e_binop(e_var("vars"), BinOp::Add, e_var("inherited"))));
    body
}

/// `$vars['name'] = <default>;`, guarded by the calling scope for a non-public property.
///
/// The default goes through a `mixed` local first: one array holds every property's default,
/// whatever its type, and a bare `null` has no storable representation of its own.
fn visible_assign(class: &ClassShape, property: &ClassProperty) -> Vec<Stmt> {
    let value = property
        .default
        .as_ref()
        .map(|default| resolve_class_references(default, class))
        .unwrap_or_else(e_null);
    let assign = vec![
        s_typed_assign(t_mixed(), "value", value),
        s_array_assign("vars", e_str(&property.name), e_var("value")),
    ];
    let declaring = symbol_key(&class.name);
    let same_class = e_binop(e_var("scope"), BinOp::StrictEq, e_str(&declaring));
    match property.visibility {
        Visibility::Public => assign,
        Visibility::Private => vec![s_if(same_class, assign, Vec::new(), None)],
        Visibility::Protected => {
            let related = e_binop(
                e_call(
                    "is_subclass_of",
                    vec![e_var("scope"), e_str(&class.name)],
                ),
                BinOp::Or,
                e_call(
                    "is_subclass_of",
                    vec![e_str(&class.name), e_var("scope")],
                ),
            );
            let in_scope = e_binop(
                same_class,
                BinOp::Or,
                e_binop(
                    e_binop(e_var("scope"), BinOp::StrictNotEq, e_str("")),
                    BinOp::And,
                    related,
                ),
            );
            vec![s_if(in_scope, assign, Vec::new(), None)]
        }
    }
}

/// Copies a default expression with `self`, `static`, `parent` and `__CLASS__` spelled as the
/// classes they name in the declaring class, so it reads the same from a global function.
fn resolve_class_references(expr: &Expr, class: &ClassShape) -> Expr {
    let receiver = |receiver: &StaticReceiver| -> StaticReceiver {
        match receiver {
            StaticReceiver::Self_ | StaticReceiver::Static => {
                StaticReceiver::Named(class_name(&class.name))
            }
            StaticReceiver::Parent => match &class.parent {
                Some(parent) => StaticReceiver::Named(class_name(parent)),
                None => StaticReceiver::Parent,
            },
            named => named.clone(),
        }
    };
    let boxed = |inner: &Expr| Box::new(resolve_class_references(inner, class));
    let kind = match &expr.kind {
        ExprKind::ScopedConstantAccess { receiver: r, name } => ExprKind::ScopedConstantAccess {
            receiver: receiver(r),
            name: name.clone(),
        },
        ExprKind::ClassConstant { receiver: r } => ExprKind::ClassConstant { receiver: receiver(r) },
        ExprKind::MagicConstant(MagicConstant::Class) => ExprKind::StringLiteral(class.name.clone()),
        ExprKind::ArrayLiteral(items) => ExprKind::ArrayLiteral(
            items.iter().map(|item| resolve_class_references(item, class)).collect(),
        ),
        ExprKind::ArrayLiteralAssoc(entries) => ExprKind::ArrayLiteralAssoc(
            entries
                .iter()
                .map(|(key, value)| {
                    (
                        resolve_class_references(key, class),
                        resolve_class_references(value, class),
                    )
                })
                .collect(),
        ),
        ExprKind::BinaryOp { left, op, right } => ExprKind::BinaryOp {
            left: boxed(left),
            op: op.clone(),
            right: boxed(right),
        },
        ExprKind::Negate(inner) => ExprKind::Negate(boxed(inner)),
        ExprKind::Not(inner) => ExprKind::Not(boxed(inner)),
        ExprKind::BitNot(inner) => ExprKind::BitNot(boxed(inner)),
        ExprKind::Spread(inner) => ExprKind::Spread(boxed(inner)),
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => ExprKind::Ternary {
            condition: boxed(condition),
            then_expr: boxed(then_expr),
            else_expr: boxed(else_expr),
        },
        ExprKind::ShortTernary { value, default } => ExprKind::ShortTernary {
            value: boxed(value),
            default: boxed(default),
        },
        ExprKind::NullCoalesce { value, default } => ExprKind::NullCoalesce {
            value: boxed(value),
            default: boxed(default),
        },
        ExprKind::ArrayAccess { array, index } => ExprKind::ArrayAccess {
            array: boxed(array),
            index: boxed(index),
        },
        ExprKind::Cast { target, expr: inner } => ExprKind::Cast {
            target: target.clone(),
            expr: boxed(inner),
        },
        other => other.clone(),
    };
    Expr::new(kind, expr.span)
}
