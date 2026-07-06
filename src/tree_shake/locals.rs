//! Purpose:
//! Stage-2.5 intra-body local-variable typing for the reachability receiver-typer. Builds a
//! per-body `local name -> LocalType` map so a receiver that is a local variable assigned from a
//! typed source (`$v = new C`, `$v = f()` with a declared class return, `$v = $this`, a copy of
//! another typed local) resolves to that class set instead of the coarse `Any` fallback.
//!
//! Called from:
//! - `crate::tree_shake::edges`: each body scan (`<main>`, function, method, closure) computes its
//!   map once and threads it into `receiver_of` through `ScanCtx::locals`.
//!
//! Key details:
//! - SOUNDNESS OVER PRECISION. The model is *join-per-body*, not flow-sensitive: a local's type is
//!   the join of EVERY assignment to it in the whole body (order-insensitive). If any assignment is
//!   dynamic/unknown, or the variable is ref-assigned (`=&`), captured by-reference into a closure,
//!   incremented, foreach/list-bound, or a `static`/`global`, the local widens to `Any`. Every
//!   parameter is seeded with its incoming value (a class hint, else `Any`) so a reassigned
//!   parameter still accounts for the original argument. Missing var = `Any` at the use site.
//! - The solve is a monotone widening fixpoint (each local only moves `bottom -> Known -> Any`, and
//!   `Known` sets only grow), so it converges in at most `#locals + constant` passes.
//! - `new C` types the local to EXACTLY `{C}` (even if `C` is external/unmodeled — then dispatch
//!   resolves to no modeled method, which is sound). Class-hint/`$this`/typed-property/call-return
//!   sources expand to `subtypes(T)` because the runtime object may be any subtype.

use std::collections::{HashMap, HashSet};

use crate::parser::ast::{Expr, ExprKind, StaticReceiver, Stmt, StmtKind, TypeExpr};

use super::reach::Analyzer;
use super::skeleton::strip_root;

/// The per-body map from a local variable name to its join-per-body type. An absent entry means the
/// variable was never given a typeable source, so its use-site type is `Any`.
pub(super) type LocalTypes = HashMap<String, LocalType>;

/// The tiny local-type lattice: a closed set of candidate classes, or the `Any` top element.
///
/// `Known` holds the receiver candidate classes (already expanded to subtypes where the source is a
/// hint/`$this`/return type, or the exact class for `new C`). `Any` is the sound fallback used
/// whenever the type cannot be cheaply and certainly determined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LocalType {
    /// A closed candidate class set for the variable.
    Known(HashSet<String>),
    /// Unknown: the receiver must fall back over instantiated classes by method name.
    Any,
}

/// One typed source contributing to a local's join: a declared type hint, an assignment RHS
/// expression, or an unconditional `Any` widen (poison from a ref/closure/foreach/etc. rebind).
enum Source<'a> {
    /// A declared type hint (parameter type or `TypedAssign` type).
    Hint(&'a TypeExpr),
    /// An assignment right-hand-side expression to evaluate.
    Expr(&'a Expr),
    /// A poison source that unconditionally widens the local to `Any`.
    Any,
}

/// The result of evaluating a single source under the current (in-progress) local map.
enum SourceEval {
    /// A resolved candidate class set.
    Known(HashSet<String>),
    /// Unknown → the local becomes `Any`.
    Any,
    /// Not yet computable this pass (depends on a local still being solved); contributes nothing.
    Bottom,
}

impl<'a> Analyzer<'a> {
    /// Computes the per-body local-variable type map for `stmts`, seeding every parameter with its
    /// incoming value type (`Hint` for a class-typed parameter, else `Any`) and every
    /// closure-captured name in `captured_any` with `Any`, then joining in all in-body assignments
    /// and solving the widening fixpoint. `enclosing` types `$this`/`self`/`parent`/`static`.
    pub(super) fn compute_local_types(
        &self,
        stmts: &'a [Stmt],
        enclosing: Option<&str>,
        params: &'a [(String, Option<TypeExpr>, Option<Expr>, bool)],
        captured_any: &[&str],
    ) -> LocalTypes {
        let mut sources: HashMap<String, Vec<Source<'a>>> = HashMap::new();
        // Every parameter carries an incoming argument value: a class hint pins it, an untyped
        // parameter is unknown (`Any`). A later reassignment joins on top of this seed, so a
        // reassigned parameter never under-approximates the original argument.
        for (name, ty, _, _) in params {
            match ty {
                Some(ty) => push(&mut sources, name, Source::Hint(ty)),
                None => push(&mut sources, name, Source::Any),
            }
        }
        // A closure body's captured variables are unknown incoming values; we do not thread the
        // enclosing types in, so they widen to `Any` (sound, if imprecise, inside closures).
        for name in captured_any {
            push(&mut sources, name, Source::Any);
        }
        for stmt in stmts {
            collect_stmt_sources(stmt, &mut sources);
        }
        self.solve_local_types(&sources, enclosing)
    }

    /// Computes the set of local names that provably hold ONLY closure values in this body.
    ///
    /// A name qualifies when it has at least one closure-flavoured binding (a closure/arrow-fn
    /// literal, or a `Closure`/`\Closure` type-hinted parameter) and NO other kind of binding —
    /// no non-closure assignment, and none of the `Any`-poison forms (reference alias, `foreach`
    /// bind, list destructure, `static`/`global`, by-reference closure capture, increment, or an
    /// untyped/non-`Closure`-typed parameter). Reuses the same source-collection walk as
    /// `compute_local_types`, so every binding form is accounted for; a missed binding could only
    /// make a name look *more* closure-valued, which is why the classifier is all-sources-agree.
    ///
    /// Used by the opaque-callable rule: an opaque invocation of a provably-closure value is bounded
    /// (the closure body was already scanned inline at its literal), so it does NOT trigger the broad
    /// callable fallback. A value we cannot prove to be a closure is treated as possibly a computed
    /// string callable and broadened (sound over-approximation).
    pub(super) fn closure_valued_locals(
        &self,
        stmts: &'a [Stmt],
        params: &'a [(String, Option<TypeExpr>, Option<Expr>, bool)],
        captured_any: &[&str],
    ) -> HashSet<String> {
        let mut sources: HashMap<String, Vec<Source<'a>>> = HashMap::new();
        for (name, ty, _, _) in params {
            match ty {
                Some(ty) => push(&mut sources, name, Source::Hint(ty)),
                None => push(&mut sources, name, Source::Any),
            }
        }
        for name in captured_any {
            push(&mut sources, name, Source::Any);
        }
        for stmt in stmts {
            collect_stmt_sources(stmt, &mut sources);
        }
        let mut out: HashSet<String> = HashSet::new();
        for (name, srcs) in &sources {
            if !srcs.is_empty()
                && srcs.iter().all(source_is_closure)
                && srcs.iter().any(source_is_closure)
            {
                out.insert(name.clone());
            }
        }
        out
    }

    /// Runs the monotone widening fixpoint over the collected sources, returning the stable local
    /// map. Each local only widens (`bottom -> Known -> Any`, `Known` sets only grow), so the loop
    /// converges well within the `#locals + 4` cap; the cap only guards against a latent bug.
    fn solve_local_types(
        &self,
        sources: &HashMap<String, Vec<Source<'a>>>,
        enclosing: Option<&str>,
    ) -> LocalTypes {
        let vars: HashSet<&str> = sources.keys().map(String::as_str).collect();
        let mut map: LocalTypes = HashMap::new();
        let cap = sources.len() + 4;
        for _ in 0..cap {
            let mut changed = false;
            for (name, srcs) in sources {
                if let Some(ty) = self.join_sources(srcs, &map, &vars, enclosing) {
                    if map.get(name) != Some(&ty) {
                        map.insert(name.clone(), ty);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        map
    }

    /// Joins a local's sources under the current map: `Any` if any source is `Any`, else the union
    /// of all `Known` class sets, or `None` (still bottom) when every source is bottom/pending.
    fn join_sources(
        &self,
        srcs: &[Source<'a>],
        map: &LocalTypes,
        vars: &HashSet<&str>,
        enclosing: Option<&str>,
    ) -> Option<LocalType> {
        let mut acc: HashSet<String> = HashSet::new();
        let mut saw_known = false;
        for src in srcs {
            match self.eval_source(src, map, vars, enclosing) {
                SourceEval::Any => return Some(LocalType::Any),
                SourceEval::Known(set) => {
                    acc.extend(set);
                    saw_known = true;
                }
                SourceEval::Bottom => {}
            }
        }
        if saw_known {
            Some(LocalType::Known(acc))
        } else {
            None
        }
    }

    /// Evaluates one source to a candidate set / `Any` / bottom under the current map.
    fn eval_source(
        &self,
        src: &Source<'a>,
        map: &LocalTypes,
        vars: &HashSet<&str>,
        enclosing: Option<&str>,
    ) -> SourceEval {
        match src {
            Source::Any => SourceEval::Any,
            Source::Hint(ty) => self.type_classes(ty),
            Source::Expr(expr) => self.eval_expr_type(expr, map, vars, enclosing),
        }
    }

    /// Types an assignment RHS expression. Only the cheaply-certain object-producing shapes yield a
    /// `Known` set; every other shape (array element, ternary, untyped property, scalar op, …) is
    /// `Any`, and a reference to a still-unsolved local is `Bottom` (retried next pass).
    fn eval_expr_type(
        &self,
        expr: &'a Expr,
        map: &LocalTypes,
        vars: &HashSet<&str>,
        enclosing: Option<&str>,
    ) -> SourceEval {
        match &expr.kind {
            // `new C` constructs EXACTLY `C` (never a subtype), even if `C` is external/unmodeled.
            ExprKind::NewObject { class_name, .. } => {
                SourceEval::Known(std::iter::once(strip_root(class_name.as_str()).to_string()).collect())
            }
            // `new self()`/`new parent()` resolve to the enclosing/parent class; `new static()`
            // is late-bound, so `scoped_receiver_class` yields the enclosing class as a sound bound.
            ExprKind::NewScopedObject { receiver, .. } => {
                match self.scoped_receiver_class(receiver, enclosing) {
                    Some(class) => {
                        SourceEval::Known(self.skeleton.subtypes(&class).into_iter().collect())
                    }
                    None => SourceEval::Any,
                }
            }
            // A runtime class-string construction could be any class → unknown.
            ExprKind::NewDynamic { .. } | ExprKind::NewDynamicObject { .. } => SourceEval::Any,
            // `$this` is the enclosing class or any subtype the runtime object could be.
            ExprKind::This => match enclosing {
                Some(class) => SourceEval::Known(self.skeleton.subtypes(class).into_iter().collect()),
                None => SourceEval::Any,
            },
            ExprKind::Variable(name) => self.eval_var(name, map, vars),
            ExprKind::FunctionCall { name, .. } => self.eval_fn_return(name.as_str()),
            ExprKind::StaticMethodCall { receiver, method, .. } => {
                self.eval_static_return(receiver, method, enclosing)
            }
            ExprKind::MethodCall { object, method, .. }
            | ExprKind::NullsafeMethodCall { object, method, .. } => {
                self.eval_method_return(object, method, map, vars, enclosing)
            }
            // `$v = ($x = new C)` yields the inner assignment's value type.
            ExprKind::Assignment { value, .. } => self.eval_expr_type(value, map, vars, enclosing),
            // `$this->prop` with a declared class property type pins to that type's subtypes.
            ExprKind::PropertyAccess { object, property }
            | ExprKind::NullsafePropertyAccess { object, property } => {
                if matches!(object.kind, ExprKind::This) {
                    if let Some(class) = enclosing {
                        if let Some(ty) = self.index.property_type(class, property) {
                            return self.type_classes(ty);
                        }
                    }
                }
                SourceEval::Any
            }
            // Everything else is not a cheaply-typeable object source → sound fallback.
            _ => SourceEval::Any,
        }
    }

    /// Looks up a variable reference in the in-progress map: a solved local yields its type, a local
    /// with sources not yet solved is `Bottom` (pending), and a variable with no sources at all is
    /// `Any` (an unknown local).
    fn eval_var(&self, name: &str, map: &LocalTypes, vars: &HashSet<&str>) -> SourceEval {
        if let Some(ty) = map.get(name) {
            return match ty {
                LocalType::Known(set) => SourceEval::Known(set.clone()),
                LocalType::Any => SourceEval::Any,
            };
        }
        if vars.contains(name) {
            SourceEval::Bottom
        } else {
            SourceEval::Any
        }
    }

    /// Types a free-function call result from its declared return type: a class/interface return
    /// expands to its subtypes; an untyped/scalar/unknown return (or an unindexed callee) is `Any`.
    fn eval_fn_return(&self, name: &str) -> SourceEval {
        match self.index.functions.get(strip_root(name)).and_then(|entry| entry.return_type) {
            Some(ty) => self.type_classes(ty),
            None => SourceEval::Any,
        }
    }

    /// Types a static-method call result from the callee's declared return type. `static::m()` is
    /// late-bound (an override could return a different class), so it soundly yields `Any`.
    fn eval_static_return(
        &self,
        receiver: &StaticReceiver,
        method: &str,
        enclosing: Option<&str>,
    ) -> SourceEval {
        if matches!(receiver, StaticReceiver::Static) {
            return SourceEval::Any;
        }
        let Some(class) = self.scoped_receiver_class(receiver, enclosing) else {
            return SourceEval::Any;
        };
        match self.index.method_return_type(&class, &method.to_ascii_lowercase()) {
            Some(ty) => self.type_classes(ty),
            None => SourceEval::Any,
        }
    }

    /// Types an instance-method call result: resolves the method over the receiver's candidate
    /// classes and unions their declared class-return types. Any receiver class whose method is
    /// unresolved or returns a non-class type collapses the whole result to `Any` (sound).
    fn eval_method_return(
        &self,
        object: &'a Expr,
        method: &str,
        map: &LocalTypes,
        vars: &HashSet<&str>,
        enclosing: Option<&str>,
    ) -> SourceEval {
        let classes = match self.eval_expr_type(object, map, vars, enclosing) {
            SourceEval::Known(set) => set,
            SourceEval::Any => return SourceEval::Any,
            SourceEval::Bottom => return SourceEval::Bottom,
        };
        let lower = method.to_ascii_lowercase();
        let mut acc: HashSet<String> = HashSet::new();
        let mut saw = false;
        for class in &classes {
            match self.index.method_return_type(class, &lower) {
                Some(ty) => match self.classes_for_type(ty) {
                    Some(set) if !set.is_empty() => {
                        acc.extend(set);
                        saw = true;
                    }
                    _ => return SourceEval::Any,
                },
                None => return SourceEval::Any,
            }
        }
        if saw {
            SourceEval::Known(acc)
        } else {
            SourceEval::Any
        }
    }

    /// Maps a declared type hint to `Known(subtypes)` for a class/interface (union members
    /// combined), or `Any` when the type is scalar/`mixed`/`object`/unresolved/external.
    fn type_classes(&self, ty: &TypeExpr) -> SourceEval {
        match self.classes_for_type(ty) {
            Some(set) if !set.is_empty() => SourceEval::Known(set),
            _ => SourceEval::Any,
        }
    }
}

/// Pushes a source onto a variable's source list, creating the entry on first sight.
fn push<'a>(sources: &mut HashMap<String, Vec<Source<'a>>>, name: &str, src: Source<'a>) {
    sources.entry(name.to_string()).or_default().push(src);
}

/// Returns `true` when a single binding source is closure-flavoured: an assignment RHS that is a
/// closure/arrow-fn literal or a first-class callable (`f(...)`, `$o->m(...)`), or a `Closure`/
/// `\Closure` type-hint. A first-class callable evaluates to a `Closure` wrapping a KNOWN target,
/// which the edge walk already enqueues where the callable value is created, so an opaque invocation
/// of it is safely bounded. A variable RHS (even one holding another closure), any other expression,
/// or the `Any` poison is treated as non-closure, so the all-sources-agree classifier in
/// `closure_valued_locals` stays conservative (never over-claims).
fn source_is_closure(src: &Source<'_>) -> bool {
    match src {
        Source::Expr(expr) => {
            matches!(expr.kind, ExprKind::Closure { .. } | ExprKind::FirstClassCallable(_))
        }
        Source::Hint(ty) => is_closure_type(ty),
        Source::Any => false,
    }
}

/// Returns `true` when a declared type hint is exactly `Closure`/`\Closure` (or its nullable form),
/// the only type that statically guarantees a closure value. `callable` is deliberately excluded:
/// a `callable` may be a string/array callable, which must stay unbounded for soundness.
fn is_closure_type(ty: &TypeExpr) -> bool {
    match ty {
        TypeExpr::Named(name) => strip_root(name.as_str()).eq_ignore_ascii_case("Closure"),
        TypeExpr::Nullable(inner) => is_closure_type(inner),
        _ => false,
    }
}

/// Collects the typed sources contributed by one statement, recursing into control-flow bodies but
/// NOT into nested declarations (separate scopes). Every variable-binding form is covered so no
/// assignment is missed — an unseen assignment would let a stale `Known` type survive (unsound).
fn collect_stmt_sources<'a>(stmt: &'a Stmt, sources: &mut HashMap<String, Vec<Source<'a>>>) {
    match &stmt.kind {
        StmtKind::Assign { name, value } => {
            push(sources, name, Source::Expr(value));
            collect_expr_sources(value, sources);
        }
        StmtKind::TypedAssign { type_expr, name, value } => {
            push(sources, name, Source::Hint(type_expr));
            push(sources, name, Source::Expr(value));
            collect_expr_sources(value, sources);
        }
        // `$t =& $src` aliases the target's storage; the alias may be rebound elsewhere → `Any`.
        StmtKind::RefAssign { target, source } => {
            push(sources, target, Source::Any);
            collect_expr_sources(source, sources);
        }
        StmtKind::RefAssignToTarget { target, source, .. } => {
            collect_expr_sources(target, sources);
            collect_expr_sources(source, sources);
        }
        // `$a[$k] = v` / `$a[] = v` mutate the array element, never rebinding `$a` to a new object.
        StmtKind::ArrayAssign { index, value, .. } => {
            collect_expr_sources(index, sources);
            collect_expr_sources(value, sources);
        }
        StmtKind::NestedArrayAssign { target, value } => {
            collect_expr_sources(target, sources);
            collect_expr_sources(value, sources);
        }
        StmtKind::ArrayPush { value, .. } => collect_expr_sources(value, sources),
        StmtKind::ListUnpack { vars, value } => {
            for var in vars {
                push(sources, var, Source::Any);
            }
            collect_expr_sources(value, sources);
        }
        // A foreach binds the value (and key) to unknown element types → `Any`.
        StmtKind::Foreach { array, key_var, value_var, body, .. } => {
            collect_expr_sources(array, sources);
            push(sources, value_var, Source::Any);
            if let Some(key) = key_var {
                push(sources, key, Source::Any);
            }
            collect_stmts_sources(body, sources);
        }
        // A `static`/`global` variable persists arbitrary state across calls → `Any`.
        StmtKind::StaticVar { name, init } => {
            push(sources, name, Source::Any);
            collect_expr_sources(init, sources);
        }
        StmtKind::Global { vars } => {
            for var in vars {
                push(sources, var, Source::Any);
            }
        }
        StmtKind::ConstDecl { value, .. } => collect_expr_sources(value, sources),
        StmtKind::Echo(expr)
        | StmtKind::Throw(expr)
        | StmtKind::ExprStmt(expr)
        | StmtKind::Return(Some(expr))
        | StmtKind::Include { path: expr, .. } => collect_expr_sources(expr, sources),
        StmtKind::If { condition, then_body, elseif_clauses, else_body } => {
            collect_expr_sources(condition, sources);
            collect_stmts_sources(then_body, sources);
            for (cond, body) in elseif_clauses {
                collect_expr_sources(cond, sources);
                collect_stmts_sources(body, sources);
            }
            if let Some(body) = else_body {
                collect_stmts_sources(body, sources);
            }
        }
        StmtKind::IfDef { then_body, else_body, .. } => {
            collect_stmts_sources(then_body, sources);
            if let Some(body) = else_body {
                collect_stmts_sources(body, sources);
            }
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { condition, body } => {
            collect_expr_sources(condition, sources);
            collect_stmts_sources(body, sources);
        }
        StmtKind::For { init, condition, update, body } => {
            if let Some(init) = init {
                collect_stmt_sources(init, sources);
            }
            if let Some(condition) = condition {
                collect_expr_sources(condition, sources);
            }
            if let Some(update) = update {
                collect_stmt_sources(update, sources);
            }
            collect_stmts_sources(body, sources);
        }
        StmtKind::Switch { subject, cases, default } => {
            collect_expr_sources(subject, sources);
            for (patterns, body) in cases {
                for pattern in patterns {
                    collect_expr_sources(pattern, sources);
                }
                collect_stmts_sources(body, sources);
            }
            if let Some(body) = default {
                collect_stmts_sources(body, sources);
            }
        }
        StmtKind::Try { try_body, catches, finally_body } => {
            collect_stmts_sources(try_body, sources);
            for catch in catches {
                // The caught variable is bound to a thrown object; a later reassignment must not
                // narrow it below the original exception, so seed `Any`.
                if let Some(var) = &catch.variable {
                    push(sources, var, Source::Any);
                }
                collect_stmts_sources(&catch.body, sources);
            }
            if let Some(body) = finally_body {
                collect_stmts_sources(body, sources);
            }
        }
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::Synthetic(body)
        | StmtKind::IncludeOnceGuard { body, .. } => collect_stmts_sources(body, sources),
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            collect_expr_sources(object, sources);
            collect_expr_sources(value, sources);
        }
        StmtKind::PropertyArrayAssign { object, index, value, .. } => {
            collect_expr_sources(object, sources);
            collect_expr_sources(index, sources);
            collect_expr_sources(value, sources);
        }
        StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => collect_expr_sources(value, sources),
        StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            collect_expr_sources(index, sources);
            collect_expr_sources(value, sources);
        }
        StmtKind::DynamicStaticPropertyWrite { property, index, value, .. } => {
            collect_expr_sources(property, sources);
            if let Some(index) = index {
                collect_expr_sources(index, sources);
            }
            collect_expr_sources(value, sources);
        }
        // Declarations (separate scopes) and leaf statements bind no local in this scope.
        _ => {}
    }
}

/// Collects sources from a statement slice.
fn collect_stmts_sources<'a>(stmts: &'a [Stmt], sources: &mut HashMap<String, Vec<Source<'a>>>) {
    for stmt in stmts {
        collect_stmt_sources(stmt, sources);
    }
}

/// Collects the typed sources contributed by one expression: nested assignments (statement-form and
/// expression-form), increments, and closure by-reference captures bind/poison locals; every other
/// compound expression is recursed into so no buried assignment is missed. Closure BODIES are a
/// separate scope and are not recursed into (only their `use (&$v)` captures poison this scope).
fn collect_expr_sources<'a>(expr: &'a Expr, sources: &mut HashMap<String, Vec<Source<'a>>>) {
    match &expr.kind {
        ExprKind::Assignment { target, value, result_target, prelude, .. } => {
            collect_stmts_sources(prelude, sources);
            match &target.kind {
                ExprKind::Variable(name) => push(sources, name, Source::Expr(value)),
                _ => collect_expr_sources(target, sources),
            }
            collect_expr_sources(value, sources);
            if let Some(result_target) = result_target {
                collect_expr_sources(result_target, sources);
            }
        }
        ExprKind::ListUnpack { vars, value } => {
            for var in vars {
                push(sources, var, Source::Any);
            }
            collect_expr_sources(value, sources);
        }
        ExprKind::PreIncrement(name)
        | ExprKind::PostIncrement(name)
        | ExprKind::PreDecrement(name)
        | ExprKind::PostDecrement(name) => push(sources, name, Source::Any),
        ExprKind::Closure { capture_refs, .. } => {
            // A by-reference capture lets the closure rebind the outer variable → widen to `Any`.
            for name in capture_refs {
                push(sources, name, Source::Any);
            }
        }
        ExprKind::FunctionCall { args, .. }
        | ExprKind::StaticMethodCall { args, .. }
        | ExprKind::ClosureCall { args, .. } => collect_args_sources(args, sources),
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            collect_expr_sources(object, sources);
            collect_args_sources(args, sources);
        }
        ExprKind::ExprCall { callee, args } => {
            collect_expr_sources(callee, sources);
            collect_args_sources(args, sources);
        }
        ExprKind::NewObject { args, .. } | ExprKind::NewScopedObject { args, .. } => {
            collect_args_sources(args, sources)
        }
        ExprKind::NewDynamic { name_expr, args } => {
            collect_expr_sources(name_expr, sources);
            collect_args_sources(args, sources);
        }
        ExprKind::NewDynamicObject { class_name, args, .. } => {
            collect_expr_sources(class_name, sources);
            collect_args_sources(args, sources);
        }
        ExprKind::BinaryOp { left, right, .. } => {
            collect_expr_sources(left, sources);
            collect_expr_sources(right, sources);
        }
        ExprKind::Print(inner)
        | ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Clone(inner)
        | ExprKind::Spread(inner)
        | ExprKind::YieldFrom(inner)
        | ExprKind::PtrCast { expr: inner, .. }
        | ExprKind::Cast { expr: inner, .. } => collect_expr_sources(inner, sources),
        ExprKind::NullCoalesce { value, default } | ExprKind::ShortTernary { value, default } => {
            collect_expr_sources(value, sources);
            collect_expr_sources(default, sources);
        }
        ExprKind::Pipe { value, callable } => {
            collect_expr_sources(value, sources);
            collect_expr_sources(callable, sources);
        }
        ExprKind::Ternary { condition, then_expr, else_expr } => {
            collect_expr_sources(condition, sources);
            collect_expr_sources(then_expr, sources);
            collect_expr_sources(else_expr, sources);
        }
        ExprKind::Match { subject, arms, default } => {
            collect_expr_sources(subject, sources);
            for (patterns, result) in arms {
                for pattern in patterns {
                    collect_expr_sources(pattern, sources);
                }
                collect_expr_sources(result, sources);
            }
            if let Some(default) = default {
                collect_expr_sources(default, sources);
            }
        }
        ExprKind::ArrayLiteral(items) => {
            for item in items {
                collect_expr_sources(item, sources);
            }
        }
        ExprKind::ArrayLiteralAssoc(items) => {
            for (key, value) in items {
                collect_expr_sources(key, sources);
                collect_expr_sources(value, sources);
            }
        }
        ExprKind::ArrayAccess { array, index } => {
            collect_expr_sources(array, sources);
            collect_expr_sources(index, sources);
        }
        ExprKind::NamedArg { value, .. } => collect_expr_sources(value, sources),
        ExprKind::PropertyAccess { object, .. } | ExprKind::NullsafePropertyAccess { object, .. } => {
            collect_expr_sources(object, sources)
        }
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            collect_expr_sources(object, sources);
            collect_expr_sources(property, sources);
        }
        ExprKind::InstanceOf { value, target } => {
            collect_expr_sources(value, sources);
            if let crate::parser::ast::InstanceOfTarget::Expr(inner) = target {
                collect_expr_sources(inner, sources);
            }
        }
        ExprKind::BufferNew { len, .. } => collect_expr_sources(len, sources),
        ExprKind::Yield { key, value } => {
            if let Some(key) = key {
                collect_expr_sources(key, sources);
            }
            if let Some(value) = value {
                collect_expr_sources(value, sources);
            }
        }
        // A first-class-callable method target carries a receiver expression that could nest an
        // assignment; mirror the edge walk and recurse into it so no binding is missed.
        ExprKind::FirstClassCallable(crate::parser::ast::CallableTarget::Method {
            object, ..
        }) => collect_expr_sources(object, sources),
        // Leaves (variables, literals, `$this`, const/class references, other callables) bind no
        // local in this scope.
        _ => {}
    }
}

/// Collects sources from a call's argument expressions.
fn collect_args_sources<'a>(args: &'a [Expr], sources: &mut HashMap<String, Vec<Source<'a>>>) {
    for arg in args {
        collect_expr_sources(arg, sources);
    }
}
