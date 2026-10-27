//! Purpose:
//! Finds user functions and methods some call site passes more positional arguments than they
//! declare, so their scope receives the hidden argument collector.
//!
//! Called from:
//! - `crate::func_args::desugar_on_compiler_stack`, before the rewriter walks the program.
//!
//! Key details:
//! - php ignores surplus arguments to a user function (`func_get_args()` still sees them);
//!   elephc refused the call. A collector makes the surplus a legal variadic tail.
//! - Functions are matched by name; methods by method name alone, since the receiver class is
//!   not known before type checking. An over-matched method only gains an unused collector.
//! - A call with a spread or a named argument is not counted: its arity is not positional.

use std::collections::HashMap;

use crate::names::php_symbol_key;
use crate::parser::ast::{Expr, ExprKind, Stmt, StmtKind};

/// The largest positional argument count seen per callee name.
#[derive(Default)]
pub(super) struct SurplusCalls {
    functions: HashMap<String, usize>,
    methods: HashMap<String, usize>,
    /// `new C(...)` counts per instantiated class; a constructor is inherited, so a count also
    /// reaches the ancestor that declares it.
    constructors: HashMap<String, usize>,
    /// Each declared class's parent, by key.
    parents: HashMap<String, String>,
}

impl SurplusCalls {
    /// Collects the call counts of the whole program.
    pub(super) fn collect(program: &mut [Stmt]) -> Self {
        let mut calls = Self::default();
        collect_parents(program, &mut calls.parents);
        crate::undefined_constants::walk_program_exprs_mut(program, &mut |expr| calls.expr(expr));
        calls
    }

    /// Returns whether a `new` of `class` or of one of its subclasses passes more arguments
    /// than the `declared` parameters of the constructor `class` declares.
    pub(super) fn constructor_needs_collector(&self, class: &str, declared: usize) -> bool {
        let class = callee_key(class);
        self.constructors.iter().any(|(instantiated, count)| {
            *count > declared && self.extends_or_is(instantiated, &class)
        })
    }

    /// Returns whether `class` is `ancestor` or inherits from it.
    fn extends_or_is(&self, class: &str, ancestor: &str) -> bool {
        let mut current = Some(class);
        let mut steps = 0;
        while let Some(name) = current {
            if name == ancestor {
                return true;
            }
            steps += 1;
            if steps > 64 {
                return false;
            }
            current = self.parents.get(name).map(String::as_str);
        }
        false
    }

    /// Returns whether a call passes `function` more arguments than its `declared` parameters.
    pub(super) fn function_needs_collector(&self, function: &str, declared: usize) -> bool {
        self.functions
            .get(&callee_key(function))
            .is_some_and(|count| *count > declared)
    }

    /// Returns whether a call passes a method of this name more than `declared` arguments.
    pub(super) fn method_needs_collector(&self, method: &str, declared: usize) -> bool {
        self.methods
            .get(&callee_key(method))
            .is_some_and(|count| *count > declared)
    }

    /// Returns whether any call site was counted.
    pub(super) fn is_empty(&self) -> bool {
        self.functions.is_empty() && self.methods.is_empty() && self.constructors.is_empty()
    }

    /// Records the call one expression node makes, if any.
    fn expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::FunctionCall { name, args } => {
                record(&mut self.functions, name.as_str(), args);
            }
            ExprKind::MethodCall { method, args, .. }
            | ExprKind::NullsafeMethodCall { method, args, .. }
            | ExprKind::StaticMethodCall { method, args, .. } => {
                record(&mut self.methods, method, args);
            }
            // `new static`/`new self` name no class before checking; those keep the arity check.
            ExprKind::NewObject { class_name, args } => {
                record(&mut self.constructors, class_name.as_str(), args);
            }
            _ => {}
        }
    }
}

/// Records every declared class's parent, through namespace and guard blocks.
fn collect_parents(stmts: &[Stmt], parents: &mut HashMap<String, String>) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::ClassDecl { name, extends: Some(parent), .. } => {
                parents.insert(callee_key(name), callee_key(parent.as_str()));
            }
            StmtKind::NamespaceBlock { body, .. }
            | StmtKind::Synthetic(body)
            | StmtKind::IncludeOnceGuard { body, .. } => collect_parents(body, parents),
            StmtKind::If { then_body, elseif_clauses, else_body, .. } => {
                collect_parents(then_body, parents);
                for (_, body) in elseif_clauses {
                    collect_parents(body, parents);
                }
                if let Some(body) = else_body {
                    collect_parents(body, parents);
                }
            }
            _ => {}
        }
    }
}

/// Records one call's positional argument count.
fn record(map: &mut HashMap<String, usize>, name: &str, args: &[Expr]) {
    let positional = args
        .iter()
        .all(|arg| !matches!(arg.kind, ExprKind::Spread(_) | ExprKind::NamedArg { .. }));
    if !positional || args.is_empty() {
        return;
    }
    let entry = map.entry(callee_key(name)).or_insert(0);
    *entry = (*entry).max(args.len());
}

/// Normalizes a callee name the way php resolves it: case-insensitive, no leading `\`.
fn callee_key(name: &str) -> String {
    php_symbol_key(name.trim_start_matches('\\'))
}
