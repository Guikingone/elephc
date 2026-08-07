//! Purpose:
//! A Rust builder for the synthetic PHP class surfaces elephc injects itself (`ext-dom`,
//! PDO, the session handlers, the image extensions). It expresses those declarations as
//! Rust data instead of PHP source text held in a `&'static str`.
//!
//! Called from:
//! - `crate::dom_prelude`, which builds the `ext-dom` surface through it.
//!
//! Key details:
//! - Produces `StmtKind::ClassDecl` nodes DIRECTLY, so the result enters the pipeline at
//!   exactly the point a parsed prelude did: collected, checked, lowered and emitted like a
//!   user class. That property is the whole reason the PHP-text form existed. A class
//!   registered straight into the checker's class map (a `FlattenedClass`) type-checks but
//!   never reaches lowering, so `new C()` dies at the backend with "constructor call to
//!   C::__construct without an emitted EIR method body" — see `codegen/lower_inst/objects.rs`.
//!   Building the AST keeps the method bodies real while removing the embedded PHP.
//! - Unused PARAMETERS are consumed by a synthesized `$_unused = …;` statement. The `$_`
//!   prefix is what exempts a name from the unused-variable warning, and these preludes are
//!   injected unconditionally, so a method that forgets the line warns on EVERY compile —
//!   `<?php echo "hi";` included. In the PHP form that line was hand-written per method and
//!   had to be kept in sync by eye; here `uses()` names the parameters the body actually
//!   reads and the rest are consumed automatically, so the hazard is removed rather than
//!   transcribed. Parameters are NOT renamed to `$_name`: PHP named arguments make a
//!   parameter's name part of the public API, and a shell has no licence to rename
//!   `setAttribute(name:, value:)`.
//! - Method-local variables should stay `$_`-prefixed for the same reason `pdo_prelude`
//!   does it: the checker resolves a method-body variable's type against top-level variables
//!   of the same name, so a user global named `$node` would otherwise clash with a plain
//!   method-local `$node`.
//! - Spans are `Span::dummy()`. A synthetic declaration has no source location, and aiming a
//!   diagnostic at a line of the compiler's own Rust would be worse than aiming it nowhere.

use crate::names::Name;
use crate::parser::ast::{
    ClassMethod, ClassProperty, Expr, ExprKind, PropertyHooks, Stmt, StmtKind, TypeExpr,
    Visibility,
};
use crate::span::Span;

// ---------------------------------------------------------------------------
// Type helpers
// ---------------------------------------------------------------------------

/// PHP's `mixed`, which the parser models as an unqualified named type rather than a
/// dedicated `TypeExpr` variant (see `parser/stmt/params.rs`).
pub fn t_mixed() -> TypeExpr {
    TypeExpr::Named(Name::unqualified("mixed"))
}

/// A class or interface type by name (`DOMNode`, `Iterator`, …).
pub fn t_class(name: &str) -> TypeExpr {
    TypeExpr::Named(Name::unqualified(name))
}

/// The nullable form `?T`.
pub fn t_nullable(inner: TypeExpr) -> TypeExpr {
    TypeExpr::Nullable(Box::new(inner))
}

// ---------------------------------------------------------------------------
// Expression / statement helpers
// ---------------------------------------------------------------------------

/// The `null` literal.
pub fn e_null() -> Expr {
    Expr::new(ExprKind::Null, Span::dummy())
}

/// A `true`/`false` literal.
pub fn e_bool(value: bool) -> Expr {
    Expr::new(ExprKind::BoolLiteral(value), Span::dummy())
}

/// `new C()` with no constructor arguments.
pub fn e_new(class_name: &str) -> Expr {
    Expr::new(
        ExprKind::NewObject {
            class_name: Name::unqualified(class_name),
            args: Vec::new(),
        },
        Span::dummy(),
    )
}

/// `return <value>;`
pub fn s_return(value: Expr) -> Stmt {
    Stmt::new(StmtKind::Return(Some(value)), Span::dummy())
}

// ---------------------------------------------------------------------------
// Methods
// ---------------------------------------------------------------------------

/// Builder for one method of a synthetic class.
///
/// The default shape is a `public` instance method with no parameters, no declared return
/// type and an empty body — the smallest thing that still lowers to a real EIR body.
pub struct MethodBuilder {
    name: String,
    params: Vec<(String, Option<TypeExpr>, Option<Expr>, bool)>,
    return_type: Option<TypeExpr>,
    body: Vec<Stmt>,
    /// Parameter names the body reads. Every other parameter gets a synthesized
    /// `$_unused = …;` consumption so the unconditional injection stays warning-free.
    used_params: Vec<String>,
}

/// Starts a `public` instance method named `name`.
pub fn method(name: &str) -> MethodBuilder {
    MethodBuilder {
        name: name.to_string(),
        params: Vec::new(),
        return_type: None,
        body: Vec::new(),
        used_params: Vec::new(),
    }
}

impl MethodBuilder {
    /// Appends a required parameter.
    pub fn param(mut self, name: &str, ty: TypeExpr) -> Self {
        self.params.push((name.to_string(), Some(ty), None, false));
        self
    }

    /// Appends an optional parameter with a default value.
    pub fn param_default(mut self, name: &str, ty: TypeExpr, default: Expr) -> Self {
        self.params
            .push((name.to_string(), Some(ty), Some(default), false));
        self
    }

    /// Declares the return type.
    pub fn returns(mut self, ty: TypeExpr) -> Self {
        self.return_type = Some(ty);
        self
    }

    /// Sets the body to a single `return <value>;`.
    pub fn returning(mut self, value: Expr) -> Self {
        self.body = vec![s_return(value)];
        self
    }

    /// Names the parameters the body actually reads, exempting them from the synthesized
    /// `$_unused` consumption. Call it whenever `returning()` hands back a parameter.
    pub fn uses(mut self, names: &[&str]) -> Self {
        self.used_params
            .extend(names.iter().map(|name| name.to_string()));
        self
    }

    /// Builds the method, prepending the `$_unused` consumption when it is needed.
    ///
    /// One unread parameter becomes `$_unused = $p;`; several become
    /// `$_unused = [$a, $b];`, matching what the hand-written PHP did.
    fn build(self) -> ClassMethod {
        let unread: Vec<Expr> = self
            .params
            .iter()
            .filter(|(name, _, _, _)| !self.used_params.iter().any(|used| used == name))
            .map(|(name, _, _, _)| Expr::var(name.clone()))
            .collect();

        let mut body = Vec::with_capacity(self.body.len() + 1);
        if !unread.is_empty() {
            let consumed = if unread.len() == 1 {
                unread.into_iter().next().expect("checked non-empty")
            } else {
                Expr::new(ExprKind::ArrayLiteral(unread), Span::dummy())
            };
            body.push(Stmt::new(
                StmtKind::ExprStmt(Expr::new(
                    ExprKind::Assignment {
                        target: Box::new(Expr::var("_unused")),
                        value: Box::new(consumed),
                        result_target: None,
                        prelude: Vec::new(),
                        conditional_value_temp: None,
                    },
                    Span::dummy(),
                )),
                Span::dummy(),
            ));
        }
        body.extend(self.body);

        let param_attributes = vec![Vec::new(); self.params.len()];
        ClassMethod {
            name: self.name,
            visibility: Visibility::Public,
            is_static: false,
            is_abstract: false,
            is_final: false,
            has_body: true,
            params: self.params,
            param_attributes,
            variadic: None,
            variadic_by_ref: false,
            variadic_type: None,
            return_type: self.return_type,
            by_ref_return: false,
            body,
            span: Span::dummy(),
            attributes: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Classes
// ---------------------------------------------------------------------------

/// Builder for one synthetic class declaration.
pub struct ClassBuilder {
    name: String,
    extends: Option<Name>,
    implements: Vec<Name>,
    properties: Vec<ClassProperty>,
    methods: Vec<ClassMethod>,
}

/// Starts a concrete (non-abstract, non-final) class named `name`.
pub fn class(name: &str) -> ClassBuilder {
    ClassBuilder {
        name: name.to_string(),
        extends: None,
        implements: Vec::new(),
        properties: Vec::new(),
        methods: Vec::new(),
    }
}

impl ClassBuilder {
    /// Sets the parent class.
    pub fn extends(mut self, parent: &str) -> Self {
        self.extends = Some(Name::unqualified(parent));
        self
    }

    /// Adds an implemented interface.
    pub fn implements(mut self, interface: &str) -> Self {
        self.implements.push(Name::unqualified(interface));
        self
    }

    /// Adds a `public` property with a declared type and an optional default.
    pub fn prop(mut self, name: &str, ty: TypeExpr, default: Option<Expr>) -> Self {
        self.properties.push(ClassProperty {
            name: name.to_string(),
            visibility: Visibility::Public,
            set_visibility: None,
            type_expr: Some(ty),
            hooks: PropertyHooks::none(),
            readonly: false,
            is_final: false,
            is_static: false,
            is_abstract: false,
            by_ref: false,
            is_promoted: false,
            default,
            span: Span::dummy(),
            attributes: Vec::new(),
        });
        self
    }

    /// Adds a method.
    pub fn method(mut self, builder: MethodBuilder) -> Self {
        self.methods.push(builder.build());
        self
    }

    /// Emits the class declaration statement.
    pub fn build(self) -> Stmt {
        Stmt::new(
            StmtKind::ClassDecl {
                name: self.name,
                extends: self.extends,
                implements: self.implements,
                is_abstract: false,
                is_final: false,
                is_readonly_class: false,
                trait_uses: Vec::new(),
                properties: self.properties,
                methods: self.methods,
                constants: Vec::new(),
            },
            Span::dummy(),
        )
    }
}
