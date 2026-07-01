//! Purpose:
//! The Stage-2 body walk: scans reachable function/method bodies (and the entry program) and
//! emits reachability edges — calls, `new`s, virtual dispatch, and literal callables — into the
//! `Analyzer`. This is the fixpoint's transfer function over one body.
//!
//! Called from:
//! - `crate::tree_shake::reach::Analyzer::run` (roots) and the worklist drain (functions/methods).
//!
//! Key details:
//! - The walk is LAZY on declarations: it never descends into a nested `class`/`function` body
//!   (those become reachable only through a call/`new` edge), but it DOES descend into closures,
//!   control flow, and every nested expression, mirroring the resolver/codegen recursion shape.
//! - `ScanCtx` threads the enclosing class (for `$this`/`self`/`static`/`parent`) and the in-scope
//!   parameter type hints (for receiver typing) plus a location label used in fallback notes.

use std::collections::HashMap;

use crate::parser::ast::{
    BinOp, CallableTarget, Expr, ExprKind, StaticReceiver, Stmt, StmtKind, TypeExpr,
};

use super::reach::{Analyzer, Receiver};

/// Per-body scan context: what `$this`/`self` resolve to, the visible parameter type hints, and a
/// human-readable label for the body being scanned (used in fallback diagnostics).
pub(super) struct ScanCtx<'a> {
    /// Enclosing class FQN for `$this`/`self`/`static`/`parent`; `None` inside a free function.
    enclosing: Option<String>,
    /// In-scope parameter (and closure-parameter) names mapped to their declared type hints.
    params: HashMap<&'a str, &'a TypeExpr>,
    /// Label such as `<main>`, `foo()`, or `Class::method` for fallback notes.
    location: String,
}

impl<'a> Analyzer<'a> {
    /// Scans the program's top-level statements as the entry "virtual body": executable roots plus
    /// wrapper blocks, skipping every declaration body (those are reached lazily through edges).
    pub(super) fn scan_root(&mut self, program: &'a [Stmt]) {
        let ctx = ScanCtx { enclosing: None, params: HashMap::new(), location: "<main>".into() };
        self.scan_stmts(program, &ctx);
    }

    /// Scans a reachable free function's body with its parameter hints in scope.
    pub(super) fn scan_function(&mut self, fqn: &str) {
        let Some(entry) = self.index.functions.get(fqn) else { return };
        let params = param_map(entry.params);
        let body = entry.body;
        let ctx = ScanCtx { enclosing: None, params, location: format!("{fqn}()") };
        self.scan_stmts(body, &ctx);
    }

    /// Scans a reachable method's body with the declaring class as `$this`'s type and the method's
    /// parameter hints in scope. A bodyless (abstract/interface) method scans nothing.
    pub(super) fn scan_method(&mut self, class: &str, lower: &str) {
        let Some(entry) = self.index.classes.get(class) else { return };
        let Some(method) = entry.methods.get(lower).copied() else { return };
        let params = param_map(&method.params);
        let ctx = ScanCtx {
            enclosing: Some(class.to_string()),
            params,
            location: format!("{class}::{}", method.name),
        };
        self.scan_stmts(&method.body, &ctx);
    }

    /// Scans a slice of statements in `ctx`.
    fn scan_stmts(&mut self, stmts: &'a [Stmt], ctx: &ScanCtx<'a>) {
        for stmt in stmts {
            self.scan_stmt(stmt, ctx);
        }
    }

    /// Scans one statement, emitting edges for its expressions and recursing into control-flow
    /// bodies. Declaration statements (class/function/interface/trait/enum) are intentionally not
    /// descended into — their members become reachable only through call/`new` edges.
    fn scan_stmt(&mut self, stmt: &'a Stmt, ctx: &ScanCtx<'a>) {
        match &stmt.kind {
            StmtKind::Echo(expr) => {
                self.scan_expr(expr, ctx);
                self.string_context(expr, ctx);
            }
            StmtKind::Throw(expr) | StmtKind::ExprStmt(expr) => self.scan_expr(expr, ctx),
            StmtKind::Return(Some(expr))
            | StmtKind::Assign { value: expr, .. }
            | StmtKind::TypedAssign { value: expr, .. }
            | StmtKind::ConstDecl { value: expr, .. }
            | StmtKind::StaticVar { init: expr, .. }
            | StmtKind::ArrayPush { value: expr, .. }
            | StmtKind::ListUnpack { value: expr, .. } => self.scan_expr(expr, ctx),
            StmtKind::RefAssign { source, .. } => self.scan_expr(source, ctx),
            StmtKind::RefAssignToTarget { target, source } => {
                self.scan_expr(target, ctx);
                self.scan_expr(source, ctx);
            }
            StmtKind::If { condition, then_body, elseif_clauses, else_body } => {
                self.scan_expr(condition, ctx);
                self.scan_stmts(then_body, ctx);
                for (cond, body) in elseif_clauses {
                    self.scan_expr(cond, ctx);
                    self.scan_stmts(body, ctx);
                }
                if let Some(body) = else_body {
                    self.scan_stmts(body, ctx);
                }
            }
            StmtKind::IfDef { then_body, else_body, .. } => {
                self.scan_stmts(then_body, ctx);
                if let Some(body) = else_body {
                    self.scan_stmts(body, ctx);
                }
            }
            StmtKind::While { condition, body } | StmtKind::DoWhile { condition, body } => {
                self.scan_expr(condition, ctx);
                self.scan_stmts(body, ctx);
            }
            StmtKind::For { init, condition, update, body } => {
                if let Some(init) = init {
                    self.scan_stmt(init, ctx);
                }
                if let Some(condition) = condition {
                    self.scan_expr(condition, ctx);
                }
                if let Some(update) = update {
                    self.scan_stmt(update, ctx);
                }
                self.scan_stmts(body, ctx);
            }
            StmtKind::Foreach { array, body, .. } => {
                self.scan_expr(array, ctx);
                self.scan_stmts(body, ctx);
            }
            StmtKind::Switch { subject, cases, default } => {
                self.scan_expr(subject, ctx);
                for (patterns, body) in cases {
                    for pattern in patterns {
                        self.scan_expr(pattern, ctx);
                    }
                    self.scan_stmts(body, ctx);
                }
                if let Some(body) = default {
                    self.scan_stmts(body, ctx);
                }
            }
            StmtKind::Try { try_body, catches, finally_body } => {
                self.scan_stmts(try_body, ctx);
                for catch in catches {
                    self.scan_stmts(&catch.body, ctx);
                }
                if let Some(body) = finally_body {
                    self.scan_stmts(body, ctx);
                }
            }
            StmtKind::NamespaceBlock { body, .. }
            | StmtKind::Synthetic(body)
            | StmtKind::IncludeOnceGuard { body, .. } => self.scan_stmts(body, ctx),
            StmtKind::ArrayAssign { index, value, .. } => {
                self.scan_expr(index, ctx);
                self.scan_expr(value, ctx);
            }
            StmtKind::NestedArrayAssign { target, value } => {
                self.scan_expr(target, ctx);
                self.scan_expr(value, ctx);
            }
            StmtKind::PropertyAssign { object, value, .. } => {
                self.scan_expr(object, ctx);
                self.scan_expr(value, ctx);
            }
            StmtKind::PropertyArrayPush { object, value, .. } => {
                self.scan_expr(object, ctx);
                self.scan_expr(value, ctx);
            }
            StmtKind::PropertyArrayAssign { object, index, value, .. } => {
                self.scan_expr(object, ctx);
                self.scan_expr(index, ctx);
                self.scan_expr(value, ctx);
            }
            StmtKind::StaticPropertyAssign { value, .. }
            | StmtKind::StaticPropertyArrayPush { value, .. } => self.scan_expr(value, ctx),
            StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
                self.scan_expr(index, ctx);
                self.scan_expr(value, ctx);
            }
            StmtKind::Include { path, .. } => self.scan_expr(path, ctx),
            // Declarations (lazy) and leaf statements with no scannable expression.
            _ => {}
        }
    }

    /// Scans one expression, emitting call/`new`/dispatch edges and recursing into every operand.
    /// Covers all `ExprKind` variants so no nested call can be silently dropped (soundness).
    fn scan_expr(&mut self, expr: &'a Expr, ctx: &ScanCtx<'a>) {
        match &expr.kind {
            ExprKind::FunctionCall { name, args } => {
                self.enqueue_function(name.as_str());
                self.handle_higher_order_call(name.as_str(), args, ctx);
                self.scan_args(args, ctx);
            }
            ExprKind::NewObject { class_name, args } => {
                // A literal `new C` constructs exactly `C`; record it only when `C` is a class we
                // know (an external/unknown class contributes no scannable body).
                let class = class_name.as_str().trim_start_matches('\\');
                if self.skeleton.classes.contains_key(class) {
                    self.instantiate(class);
                }
                self.scan_args(args, ctx);
            }
            ExprKind::NewScopedObject { receiver, args } => {
                if let Some(class) = self.scoped_receiver_class(receiver, ctx.enclosing.as_deref()) {
                    self.instantiate(&class);
                }
                self.scan_args(args, ctx);
            }
            ExprKind::NewDynamic { name_expr, args } => {
                self.instantiate_all(format!("new $dynamic at {}", ctx.location));
                self.scan_expr(name_expr, ctx);
                self.scan_args(args, ctx);
            }
            ExprKind::NewDynamicObject { class_name, fallback_class, required_parent, args } => {
                self.instantiate_subtypes(
                    required_parent.as_str(),
                    fallback_class.as_str(),
                    format!("new-by-name (parent {}) at {}", required_parent.as_str(), ctx.location),
                );
                self.scan_expr(class_name, ctx);
                self.scan_args(args, ctx);
            }
            ExprKind::StaticMethodCall { receiver, method, args } => {
                self.handle_static_call(receiver, method, ctx);
                self.scan_args(args, ctx);
            }
            ExprKind::MethodCall { object, method, args }
            | ExprKind::NullsafeMethodCall { object, method, args } => {
                self.handle_method_call(object, method, ctx);
                self.scan_expr(object, ctx);
                self.scan_args(args, ctx);
            }
            ExprKind::FirstClassCallable(target) => self.handle_callable_target(target, ctx),
            ExprKind::ExprCall { callee, args } => {
                // A closure literal is invoked with its own body (already scanned by recursion);
                // any other callee is a dynamic invocation target → sound `__invoke` fallback.
                if !matches!(callee.kind, ExprKind::Closure { .. }) {
                    self.add_named_fallback(
                        "__invoke",
                        format!("dynamic call target ()(...) at {}", ctx.location),
                    );
                }
                self.scan_expr(callee, ctx);
                self.scan_args(args, ctx);
            }
            ExprKind::ClosureCall { args, .. } => {
                self.add_named_fallback(
                    "__invoke",
                    format!("dynamic call $var(...) at {}", ctx.location),
                );
                self.scan_args(args, ctx);
            }
            ExprKind::Closure { body, params, .. } => {
                let child = self.closure_ctx(params, ctx);
                self.scan_stmts(body, &child);
            }
            ExprKind::BinaryOp { left, op, right } => {
                if matches!(op, BinOp::Concat) {
                    self.string_context(left, ctx);
                    self.string_context(right, ctx);
                }
                self.scan_expr(left, ctx);
                self.scan_expr(right, ctx);
            }
            ExprKind::Print(inner) => {
                self.scan_expr(inner, ctx);
                self.string_context(inner, ctx);
            }
            ExprKind::InstanceOf { value, target } => {
                self.scan_expr(value, ctx);
                if let crate::parser::ast::InstanceOfTarget::Expr(inner) = target {
                    self.scan_expr(inner, ctx);
                }
            }
            ExprKind::Negate(inner)
            | ExprKind::Not(inner)
            | ExprKind::BitNot(inner)
            | ExprKind::Throw(inner)
            | ExprKind::ErrorSuppress(inner)
            | ExprKind::Clone(inner)
            | ExprKind::Spread(inner)
            | ExprKind::YieldFrom(inner)
            | ExprKind::PtrCast { expr: inner, .. }
            | ExprKind::Cast { expr: inner, .. } => self.scan_expr(inner, ctx),
            ExprKind::NullCoalesce { value, default } | ExprKind::ShortTernary { value, default } => {
                self.scan_expr(value, ctx);
                self.scan_expr(default, ctx);
            }
            ExprKind::Pipe { value, callable } => {
                self.scan_expr(value, ctx);
                self.scan_expr(callable, ctx);
            }
            ExprKind::Assignment { target, value, result_target, prelude, .. } => {
                self.scan_stmts(prelude, ctx);
                self.scan_expr(target, ctx);
                self.scan_expr(value, ctx);
                if let Some(result_target) = result_target {
                    self.scan_expr(result_target, ctx);
                }
            }
            ExprKind::ListUnpack { value, .. } => self.scan_expr(value, ctx),
            ExprKind::ArrayLiteral(items) => {
                for item in items {
                    self.scan_expr(item, ctx);
                }
            }
            ExprKind::ArrayLiteralAssoc(items) => {
                for (key, value) in items {
                    self.scan_expr(key, ctx);
                    self.scan_expr(value, ctx);
                }
            }
            ExprKind::Match { subject, arms, default } => {
                self.scan_expr(subject, ctx);
                for (patterns, result) in arms {
                    for pattern in patterns {
                        self.scan_expr(pattern, ctx);
                    }
                    self.scan_expr(result, ctx);
                }
                if let Some(default) = default {
                    self.scan_expr(default, ctx);
                }
            }
            ExprKind::ArrayAccess { array, index } => {
                self.scan_expr(array, ctx);
                self.scan_expr(index, ctx);
            }
            ExprKind::Ternary { condition, then_expr, else_expr } => {
                self.scan_expr(condition, ctx);
                self.scan_expr(then_expr, ctx);
                self.scan_expr(else_expr, ctx);
            }
            ExprKind::NamedArg { value, .. } => self.scan_expr(value, ctx),
            ExprKind::PropertyAccess { object, .. }
            | ExprKind::NullsafePropertyAccess { object, .. } => self.scan_expr(object, ctx),
            ExprKind::DynamicPropertyAccess { object, property }
            | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
                self.scan_expr(object, ctx);
                self.scan_expr(property, ctx);
            }
            ExprKind::BufferNew { len, .. } => self.scan_expr(len, ctx),
            ExprKind::Yield { key, value } => {
                if let Some(key) = key {
                    self.scan_expr(key, ctx);
                }
                if let Some(value) = value {
                    self.scan_expr(value, ctx);
                }
            }
            // Leaves and reference-only nodes with no nested call to follow.
            _ => {}
        }
    }

    /// Scans a call's arguments and additionally resolves any literal callable arguments
    /// (`'C::m'`, `['C'|$obj, 'm']`, or a bare string naming a known function).
    fn scan_args(&mut self, args: &'a [Expr], ctx: &ScanCtx<'a>) {
        for arg in args {
            self.scan_callable_literal(arg, ctx);
            self.scan_expr(arg, ctx);
        }
    }

    /// Resolves a literal callable expression into edges: `"C::m"` static calls, a two-element
    /// `[receiver, "m"]` array, or a bare string that names a declared free function. Non-literal
    /// callables are handled elsewhere (or fall back), so this only fires on statically-visible ones.
    fn scan_callable_literal(&mut self, arg: &'a Expr, ctx: &ScanCtx<'a>) {
        match &arg.kind {
            ExprKind::StringLiteral(text) => {
                if let Some((class, method)) = text.split_once("::") {
                    let class = class.trim_start_matches('\\');
                    if self.skeleton.classes.contains_key(class) {
                        let set = std::iter::once(class.to_string()).collect();
                        self.enqueue_over_classes(&set, &method.to_ascii_lowercase());
                    }
                } else {
                    let fqn = text.trim_start_matches('\\');
                    if self.index.functions.contains_key(fqn) {
                        self.enqueue_function(fqn);
                    }
                }
            }
            ExprKind::ArrayLiteral(items) if items.len() == 2 => {
                if let ExprKind::StringLiteral(method) = &items[1].kind {
                    let lower = method.to_ascii_lowercase();
                    match &items[0].kind {
                        ExprKind::StringLiteral(class) => {
                            let class = class.trim_start_matches('\\');
                            if self.skeleton.classes.contains_key(class) {
                                let set = std::iter::once(class.to_string()).collect();
                                self.enqueue_over_classes(&set, &lower);
                            }
                        }
                        _ => match self.receiver_of(&items[0], ctx.enclosing.as_deref(), &ctx.params)
                        {
                            Receiver::Bound { key, classes } => {
                                self.add_typed_site(key, classes, &lower)
                            }
                            Receiver::Any => self.add_named_fallback(
                                &lower,
                                format!("literal [obj,'{method}'] callable at {}", ctx.location),
                            ),
                        },
                    }
                }
            }
            _ => {}
        }
    }

    /// Handles higher-order builtins (`call_user_func`, `array_map`, `usort`, …): a callable
    /// argument that is a literal (`'C::m'`, `[obj,'m']`), a closure, or a first-class callable is
    /// resolved by the normal walk; any OTHER (opaque, computed) callable has an unknown method
    /// name, so it triggers the sound dynamic fallback (keep every method of every instantiated
    /// class). This is the one place a variable-held callable is accounted for.
    fn handle_higher_order_call(&mut self, name: &str, args: &'a [Expr], ctx: &ScanCtx<'a>) {
        let canonical = name.trim_start_matches('\\').to_ascii_lowercase();
        for &index in callable_arg_indices(&canonical) {
            let Some(arg) = args.get(index) else { continue };
            if !is_resolvable_callable(arg) {
                self.add_dynamic_fallback(format!(
                    "opaque callable arg to {canonical}() at {}",
                    ctx.location
                ));
            }
        }
    }

    /// Emits the edge for a static method call, mapping `self`/`parent`/`static`/named receivers
    /// to the class(es) whose method is resolved. `static::m` fans out over subtypes (late binding).
    fn handle_static_call(&mut self, receiver: &StaticReceiver, method: &str, ctx: &ScanCtx<'a>) {
        let lower = method.to_ascii_lowercase();
        match receiver {
            StaticReceiver::Static => {
                if let Some(class) = ctx.enclosing.as_deref() {
                    let set = self.skeleton.subtypes(class).into_iter().collect();
                    self.enqueue_over_classes(&set, &lower);
                }
            }
            _ => {
                if let Some(class) = self.scoped_receiver_class(receiver, ctx.enclosing.as_deref()) {
                    let set = std::iter::once(class).collect();
                    self.enqueue_over_classes(&set, &lower);
                }
            }
        }
    }

    /// Emits the edge for an instance method call: pins the receiver's class set with the cheap
    /// receiver-typing helper and either records a typed call site or an unknown-receiver fallback.
    fn handle_method_call(&mut self, object: &'a Expr, method: &str, ctx: &ScanCtx<'a>) {
        let lower = method.to_ascii_lowercase();
        match self.receiver_of(object, ctx.enclosing.as_deref(), &ctx.params) {
            Receiver::Bound { key, classes } => self.add_typed_site(key, classes, &lower),
            Receiver::Any => self.add_named_fallback(
                &lower,
                format!("untyped receiver ->{method}() at {}", ctx.location),
            ),
        }
    }

    /// Resolves a first-class-callable target (`f(...)`, `C::m(...)`, `$o->m(...)`) into edges,
    /// treating it as a potential invocation of the underlying function/method (sound).
    fn handle_callable_target(&mut self, target: &'a CallableTarget, ctx: &ScanCtx<'a>) {
        match target {
            CallableTarget::Function(name) => self.enqueue_function(name.as_str()),
            CallableTarget::StaticMethod { receiver, method } => {
                self.handle_static_call(receiver, method, ctx)
            }
            CallableTarget::Method { object, method } => {
                self.handle_method_call(object, method, ctx);
                self.scan_expr(object, ctx);
            }
        }
    }

    /// If `expr` is a receiver typeable to classes that may define `__toString`, records a typed
    /// `__toString` call site (object-in-string-context). Unknown receivers are skipped, as the
    /// spec permits, to avoid a broad `__toString` fallback on every string/echo.
    fn string_context(&mut self, expr: &'a Expr, ctx: &ScanCtx<'a>) {
        if let Receiver::Bound { key, classes } =
            self.receiver_of(expr, ctx.enclosing.as_deref(), &ctx.params)
        {
            self.add_typed_site(format!("{key}#tostring"), classes, "__tostring");
        }
    }

    /// Builds a child scan context for a closure body: keeps the enclosing class and outer
    /// parameter hints, then overlays the closure's own parameter hints (which shadow captures).
    fn closure_ctx(
        &self,
        params: &'a [(String, Option<TypeExpr>, Option<Expr>, bool)],
        ctx: &ScanCtx<'a>,
    ) -> ScanCtx<'a> {
        let mut child = ScanCtx {
            enclosing: ctx.enclosing.clone(),
            params: ctx.params.clone(),
            location: format!("{} closure", ctx.location),
        };
        for (name, ty, _, _) in params {
            if let Some(ty) = ty {
                child.params.insert(name.as_str(), ty);
            }
        }
        child
    }
}

/// Builds a parameter-name → type-hint map from a parameter list, skipping untyped parameters.
fn param_map(
    params: &[(String, Option<TypeExpr>, Option<Expr>, bool)],
) -> HashMap<&str, &TypeExpr> {
    let mut map = HashMap::new();
    for (name, ty, _, _) in params {
        if let Some(ty) = ty {
            map.insert(name.as_str(), ty);
        }
    }
    map
}

/// Returns the argument positions that carry a callable for a known higher-order builtin (by its
/// canonical lowercase name), or an empty slice for any other function. Used to spot opaque
/// callables that must trigger the dynamic-dispatch fallback.
fn callable_arg_indices(name: &str) -> &'static [usize] {
    match name {
        "call_user_func" | "call_user_func_array" | "forward_static_call"
        | "forward_static_call_array" | "register_shutdown_function" | "register_tick_function"
        | "spl_autoload_register" => &[0],
        "array_map" => &[0],
        "array_filter" | "array_reduce" | "usort" | "uasort" | "uksort" | "array_walk"
        | "array_walk_recursive" | "preg_replace_callback" | "iterator_apply" => &[1],
        "preg_replace_callback_array" => &[0],
        _ => &[],
    }
}

/// Returns `true` when a callable argument is statically resolvable by the ordinary walk — a
/// closure literal, a first-class callable, or a literal string/array callable — so it does NOT
/// need the dynamic fallback. Any other (computed) expression is treated as an opaque callable.
fn is_resolvable_callable(arg: &Expr) -> bool {
    matches!(
        arg.kind,
        ExprKind::Closure { .. }
            | ExprKind::FirstClassCallable(_)
            | ExprKind::StringLiteral(_)
            | ExprKind::ArrayLiteral(_)
            | ExprKind::Null
    )
}
