//! Purpose:
//! Hoists expression-position `include`/`require` (`ExprKind::IncludeValue`) that appear NESTED
//! inside a larger expression (e.g. `self::$x ??= require F;`, `$y = 10 + require F;`) out into
//! fresh temporaries evaluated as ordinary statements before the owning statement.
//!
//! Called from:
//! - `crate::resolver::engine::resolve_stmts()`, after the direct-capture fast-path
//!   (`try_expand_value_include`) and before generic expression resolution.
//!
//! Key details:
//! - Reuses `engine_includes::expand_value_include` with `IncludeValueCapture::Assign(temp)`;
//!   this module never re-implements include inlining, it only decides WHERE the include runs.
//! - It rewrites only a statement's DIRECT expression fields. Nested statement bodies (branch
//!   bodies, loop bodies, try/catch, declarations) and isolated expression bodies (closure
//!   bodies, `Assignment.prelude`) are left untouched: `resolve_stmts`/`resolve_isolated`
//!   re-enter this hoister (and the fast-path) at those scopes.
//! - Coverage mirrors `contains::expr_has_includes`/`stmt_has_includes` so the detector
//!   (`contains::stmt_has_value_include`) and this rewriter agree: every expression position the
//!   detector flags, the hoister eliminates.
//! - SEMANTICS (intentional): hoisting evaluates the include EAGERLY, before the statement. For
//!   `self::$x ??= require F` the include therefore runs even when `self::$x` is already set (the
//!   `??=` still short-circuits the ASSIGNMENT — only the evaluation is eager). This is correct
//!   for pure data-file includes (`return [ ...array... ];`, no side effects: eager re-evaluation
//!   only rebuilds a value the `??=` then discards) and consistent with elephc's compile-time
//!   include-inlining model, where an include is always inlined at its source location and
//!   expression-level runtime conditionality is not representable. Lazy/conditional include
//!   evaluation is intentionally out of scope.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::errors::CompileError;
use crate::parser::ast::{CallableTarget, Expr, ExprKind, InstanceOfTarget, Stmt, StmtKind};

use super::discovery::FunctionVariantRegistry;
use super::engine_includes::{
    degraded_dynamic_include_expr, expand_value_include, IncludeValueCapture,
};
use super::state::ResolveState;

/// Process-global counter producing unique hidden temporary names for hoisted value-includes.
/// Uses a distinct `__elephc_incval_` prefix from `expand_value_include`'s own `__elephc_inc_`
/// temporaries so the two name spaces can never collide.
static HOIST_VALUE_INCLUDE_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Threaded resolver state shared across the recursive value-include hoist walk.
///
/// Bundling the mutable resolver context here keeps the recursive `hoist_stmt`/`hoist_expr`
/// walkers readable (they would otherwise thread seven parameters through every call, like the
/// sibling `exprs.rs` walker). `hoisted` accumulates the include-expansion statements that must
/// run before the statement currently being rewritten.
struct HoistCtx<'a> {
    /// Statements produced by expanding hoisted value-includes; prepended before the owner.
    hoisted: &'a mut Vec<Stmt>,
    /// Base directory used to resolve relative include paths.
    base_dir: &'a Path,
    /// Set of files already claimed by an include-once/require-once guard.
    declared_once: &'a mut HashSet<PathBuf>,
    /// Current include chain, used for cycle detection.
    include_chain: &'a mut Vec<PathBuf>,
    /// Shared resolver state (namespace, constants, imports).
    state: &'a mut ResolveState,
    /// Registry of include-loaded function variants.
    function_variants: &'a FunctionVariantRegistry,
}

/// Rewrites `stmt` so every value-include in one of its DIRECT expression fields is replaced by a
/// reference to a fresh temporary, appending the include-expansion statements to `hoisted` in
/// source (evaluation) order. Nested statement bodies are left unchanged for the caller's
/// recursive resolution to handle.
///
/// This is the module's only public entry point; the recursion runs through a `HoistCtx`.
pub(super) fn hoist_value_includes_in_stmt(
    stmt: Stmt,
    hoisted: &mut Vec<Stmt>,
    base_dir: &Path,
    declared_once: &mut HashSet<PathBuf>,
    include_chain: &mut Vec<PathBuf>,
    state: &mut ResolveState,
    function_variants: &FunctionVariantRegistry,
) -> Result<Stmt, CompileError> {
    let mut ctx = HoistCtx {
        hoisted,
        base_dir,
        declared_once,
        include_chain,
        state,
        function_variants,
    };
    ctx.hoist_stmt(stmt)
}

impl HoistCtx<'_> {
    /// Hoists every value-include out of `stmt`'s direct expression fields into `self.hoisted`,
    /// returning the statement with those includes replaced by temporary references.
    ///
    /// Only fields that `resolve_stmt_exprs` resolves eagerly at this scope are rewritten
    /// (assignment/echo/return values, control-flow conditions/subjects/case values, etc.).
    /// Nested statement bodies (branch/loop/try bodies, declaration bodies) and `For` init/update
    /// statements are passed through unchanged: `resolve_stmts`/`resolve_isolated` re-enter this
    /// hoister at their own scope. Statement-level `Include` is likewise left for
    /// `resolve_include_stmt`.
    fn hoist_stmt(&mut self, stmt: Stmt) -> Result<Stmt, CompileError> {
        let span = stmt.span;
        let attributes = stmt.attributes.clone();
        let kind = match stmt.kind {
            StmtKind::Echo(expr) => StmtKind::Echo(self.hoist_expr(expr)?),
            StmtKind::Throw(expr) => StmtKind::Throw(self.hoist_expr(expr)?),
            StmtKind::ExprStmt(expr) => StmtKind::ExprStmt(self.hoist_expr(expr)?),
            StmtKind::Return(expr) => StmtKind::Return(match expr {
                Some(expr) => Some(self.hoist_expr(expr)?),
                None => None,
            }),
            StmtKind::Assign { name, value } => StmtKind::Assign {
                name,
                value: self.hoist_expr(value)?,
            },
            StmtKind::TypedAssign {
                type_expr,
                name,
                value,
            } => StmtKind::TypedAssign {
                type_expr,
                name,
                value: self.hoist_expr(value)?,
            },
            StmtKind::ConstDecl { name, value } => StmtKind::ConstDecl {
                name,
                value: self.hoist_expr(value)?,
            },
            StmtKind::ListUnpack { vars, value } => StmtKind::ListUnpack {
                vars,
                value: self.hoist_expr(value)?,
            },
            StmtKind::StaticVar { name, init } => StmtKind::StaticVar {
                name,
                init: self.hoist_expr(init)?,
            },
            StmtKind::ArrayPush { array, value } => StmtKind::ArrayPush {
                array,
                value: self.hoist_expr(value)?,
            },
            StmtKind::ArrayAssign {
                array,
                index,
                value,
            } => StmtKind::ArrayAssign {
                array,
                index: self.hoist_expr(index)?,
                value: self.hoist_expr(value)?,
            },
            StmtKind::NestedArrayAssign { target, value } => StmtKind::NestedArrayAssign {
                target: self.hoist_expr(target)?,
                value: self.hoist_expr(value)?,
            },
            StmtKind::StaticPropertyAssign {
                receiver,
                property,
                value,
            } => StmtKind::StaticPropertyAssign {
                receiver,
                property,
                value: self.hoist_expr(value)?,
            },
            StmtKind::StaticPropertyArrayPush {
                receiver,
                property,
                value,
            } => StmtKind::StaticPropertyArrayPush {
                receiver,
                property,
                value: self.hoist_expr(value)?,
            },
            StmtKind::StaticPropertyArrayAssign {
                receiver,
                property,
                index,
                value,
            } => StmtKind::StaticPropertyArrayAssign {
                receiver,
                property,
                index: self.hoist_expr(index)?,
                value: self.hoist_expr(value)?,
            },
            StmtKind::DynamicStaticPropertyWrite {
                receiver,
                property,
                index,
                append,
                value,
            } => StmtKind::DynamicStaticPropertyWrite {
                receiver,
                property: self.hoist_boxed(property)?,
                index: index.map(|i| self.hoist_expr(i)).transpose()?,
                append,
                value: self.hoist_expr(value)?,
            },
            StmtKind::PropertyAssign {
                object,
                property,
                value,
            } => StmtKind::PropertyAssign {
                object: self.hoist_boxed(object)?,
                property,
                value: self.hoist_expr(value)?,
            },
            StmtKind::PropertyArrayPush {
                object,
                property,
                value,
            } => StmtKind::PropertyArrayPush {
                object: self.hoist_boxed(object)?,
                property,
                value: self.hoist_expr(value)?,
            },
            // Mirror `contains.rs`: only `index`/`value` are scanned for a `PropertyArrayAssign`
            // (its `object` receiver is skipped there via `..`), so match that coverage here.
            StmtKind::PropertyArrayAssign {
                object,
                property,
                index,
                value,
            } => StmtKind::PropertyArrayAssign {
                object,
                property,
                index: self.hoist_expr(index)?,
                value: self.hoist_expr(value)?,
            },
            StmtKind::If {
                condition,
                then_body,
                elseif_clauses,
                else_body,
            } => {
                let condition = self.hoist_expr(condition)?;
                let mut new_clauses = Vec::with_capacity(elseif_clauses.len());
                for (clause_condition, clause_body) in elseif_clauses {
                    new_clauses.push((self.hoist_expr(clause_condition)?, clause_body));
                }
                StmtKind::If {
                    condition,
                    then_body,
                    elseif_clauses: new_clauses,
                    else_body,
                }
            }
            StmtKind::While { condition, body } => StmtKind::While {
                condition: self.hoist_expr(condition)?,
                body,
            },
            StmtKind::DoWhile { body, condition } => StmtKind::DoWhile {
                body,
                condition: self.hoist_expr(condition)?,
            },
            StmtKind::For {
                init,
                condition,
                update,
                body,
            } => StmtKind::For {
                init,
                condition: match condition {
                    Some(condition) => Some(self.hoist_expr(condition)?),
                    None => None,
                },
                update,
                body,
            },
            StmtKind::Foreach {
                array,
                key_var,
                value_var,
                value_by_ref,
                body,
            } => StmtKind::Foreach {
                array: self.hoist_expr(array)?,
                key_var,
                value_var,
                value_by_ref,
                body,
            },
            StmtKind::Switch {
                subject,
                cases,
                default,
            } => {
                let subject = self.hoist_expr(subject)?;
                let mut new_cases = Vec::with_capacity(cases.len());
                for (values, body) in cases {
                    new_cases.push((self.hoist_exprs(values)?, body));
                }
                StmtKind::Switch {
                    subject,
                    cases: new_cases,
                    default,
                }
            }
            // Everything else has no direct expression field to hoist here: statement-level
            // includes are handled by `resolve_include_stmt`; nested bodies, declarations, and
            // `RefAssign*`/`For` init-update statements are handled by recursive resolution
            // (`resolve_isolated`/`resolve_stmt_exprs`). `RefAssignToTarget` is excluded to match
            // `contains.rs`, which does not flag it as include-bearing.
            other @ (StmtKind::RefAssign { .. }
            | StmtKind::RefAssignToTarget { .. }
            | StmtKind::IfDef { .. }
            | StmtKind::Include { .. }
            | StmtKind::IncludeOnceMark { .. }
            | StmtKind::IncludeOnceGuard { .. }
            | StmtKind::Synthetic(_)
            | StmtKind::Try { .. }
            | StmtKind::Break(_)
            | StmtKind::Continue(_)
            | StmtKind::Goto(_)
            | StmtKind::Label(_)
            | StmtKind::NamespaceDecl { .. }
            | StmtKind::NamespaceBlock { .. }
            | StmtKind::UseDecl { .. }
            | StmtKind::FunctionDecl { .. }
            | StmtKind::FunctionVariantGroup { .. }
            | StmtKind::FunctionVariantMark { .. }
            | StmtKind::Global { .. }
            | StmtKind::ClassDecl { .. }
            | StmtKind::EnumDecl { .. }
            | StmtKind::PackedClassDecl { .. }
            | StmtKind::InterfaceDecl { .. }
            | StmtKind::TraitDecl { .. }
            | StmtKind::ExternFunctionDecl { .. }
            | StmtKind::ExternClassDecl { .. }
            | StmtKind::ExternGlobalDecl { .. }) => other,
        };
        Ok(Stmt::with_attributes(kind, span, attributes))
    }

    /// Recursively rewrites `expr`, replacing every nested `ExprKind::IncludeValue` with a fresh
    /// temporary whose expansion is appended to `self.hoisted`. Children are recursed left-to-right
    /// so hoisted statements are emitted in PHP evaluation order.
    ///
    /// Recursion covers exactly the include-bearing positions of `contains::expr_has_includes`,
    /// except closure bodies/parameter defaults and `Assignment.prelude`, which `resolve_expr`
    /// resolves in isolation (re-entering this hoister at their own scope). Leaf expressions and
    /// closures are returned unchanged.
    fn hoist_expr(&mut self, expr: Expr) -> Result<Expr, CompileError> {
        let span = expr.span;
        let kind = match expr.kind {
            // Main-side dynamic call/class-name forms: hoist include-bearing children.
            ExprKind::NullsafeDynamicMethodCall { object, method, args } => {
                ExprKind::NullsafeDynamicMethodCall {
                    object: Box::new(self.hoist_expr(*object)?),
                    method: Box::new(self.hoist_expr(*method)?),
                    args: args
                        .into_iter()
                        .map(|arg| self.hoist_expr(arg))
                        .collect::<Result<Vec<_>, _>>()?,
                }
            }
            ExprKind::ObjectClassName { object } => ExprKind::ObjectClassName {
                object: Box::new(self.hoist_expr(*object)?),
            },
            ExprKind::IncludeValue {
                path,
                once,
                required,
            } => {
                // Degraded runtime-dynamic path under lenient lowering: there is no file to
                // inline, only a runtime fatal. Rewrite it IN PLACE as a diverging expression
                // instead of hoisting anything, so the fatal keeps the include's real program
                // point. Hoisting it would move the path expression above the owning statement —
                // wrong whenever that statement is what defines the path variable, as in
                // `if (!$file = …) { … } elseif (false === include $file) { … }`, where the
                // hoisted stub reads `$file` before its assignment.
                if let Some(diverging) =
                    degraded_dynamic_include_expr(&path, span, self.state)
                {
                    return Ok(diverging);
                }
                // Terminal case: allocate a fresh temp, expand the include so it runs in the
                // caller's scope and assigns its top-level return value to that temp, and return a
                // reference to the temp in place of the include.
                let temp = format!(
                    "__elephc_incval_{}",
                    HOIST_VALUE_INCLUDE_COUNTER.fetch_add(1, Ordering::Relaxed)
                );
                let expanded = expand_value_include(
                    span,
                    &path,
                    once,
                    required,
                    IncludeValueCapture::Assign(temp.clone()),
                    self.base_dir,
                    self.declared_once,
                    self.include_chain,
                    self.state,
                    self.function_variants,
                )?;
                self.hoisted.extend(expanded);
                ExprKind::Variable(temp)
            }
            ExprKind::BinaryOp { left, op, right } => ExprKind::BinaryOp {
                left: self.hoist_boxed(left)?,
                op,
                right: self.hoist_boxed(right)?,
            },
            ExprKind::InstanceOf { value, target } => ExprKind::InstanceOf {
                value: self.hoist_boxed(value)?,
                target: match target {
                    InstanceOfTarget::Name(name) => InstanceOfTarget::Name(name),
                    InstanceOfTarget::Expr(expr) => InstanceOfTarget::Expr(self.hoist_boxed(expr)?),
                },
            },
            ExprKind::Negate(inner) => ExprKind::Negate(self.hoist_boxed(inner)?),
            ExprKind::Not(inner) => ExprKind::Not(self.hoist_boxed(inner)?),
            ExprKind::BitNot(inner) => ExprKind::BitNot(self.hoist_boxed(inner)?),
            ExprKind::Throw(inner) => ExprKind::Throw(self.hoist_boxed(inner)?),
            ExprKind::ErrorSuppress(inner) => ExprKind::ErrorSuppress(self.hoist_boxed(inner)?),
            ExprKind::Print(inner) => ExprKind::Print(self.hoist_boxed(inner)?),
            ExprKind::Spread(inner) => ExprKind::Spread(self.hoist_boxed(inner)?),
            ExprKind::Clone(inner) => ExprKind::Clone(self.hoist_boxed(inner)?),
            ExprKind::PtrCast { target_type, expr } => ExprKind::PtrCast {
                target_type,
                expr: self.hoist_boxed(expr)?,
            },
            ExprKind::BufferNew { element_type, len } => ExprKind::BufferNew {
                element_type,
                len: self.hoist_boxed(len)?,
            },
            ExprKind::NullCoalesce { value, default } => ExprKind::NullCoalesce {
                value: self.hoist_boxed(value)?,
                default: self.hoist_boxed(default)?,
            },
            ExprKind::ArrayAccess { array, index } => ExprKind::ArrayAccess {
                array: self.hoist_boxed(array)?,
                index: self.hoist_boxed(index)?,
            },
            ExprKind::ShortTernary { value, default } => ExprKind::ShortTernary {
                value: self.hoist_boxed(value)?,
                default: self.hoist_boxed(default)?,
            },
            ExprKind::Pipe { value, callable } => ExprKind::Pipe {
                value: self.hoist_boxed(value)?,
                callable: self.hoist_boxed(callable)?,
            },
            ExprKind::ListUnpack { vars, value } => ExprKind::ListUnpack {
                vars,
                value: self.hoist_boxed(value)?,
            },
            ExprKind::Assignment {
                target,
                value,
                result_target,
                prelude,
                conditional_value_temp,
            } => ExprKind::Assignment {
                target: self.hoist_boxed(target)?,
                value: self.hoist_boxed(value)?,
                result_target: match result_target {
                    Some(result_target) => Some(self.hoist_boxed(result_target)?),
                    None => None,
                },
                // `prelude` is a nested statement list resolved in isolation by `resolve_expr`;
                // hoisting it here would run the include in the wrong scope, so leave it.
                prelude,
                conditional_value_temp,
            },
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => ExprKind::Ternary {
                condition: self.hoist_boxed(condition)?,
                then_expr: self.hoist_boxed(then_expr)?,
                else_expr: self.hoist_boxed(else_expr)?,
            },
            ExprKind::Cast { target, expr } => ExprKind::Cast {
                target,
                expr: self.hoist_boxed(expr)?,
            },
            ExprKind::NamedArg { name, value } => ExprKind::NamedArg {
                name,
                value: self.hoist_boxed(value)?,
            },
            ExprKind::FunctionCall { name, args } => ExprKind::FunctionCall {
                name,
                args: self.hoist_exprs(args)?,
            },
            ExprKind::ClosureCall { var, args } => ExprKind::ClosureCall {
                var,
                args: self.hoist_exprs(args)?,
            },
            ExprKind::StaticMethodCall {
                receiver,
                method,
                args,
            } => ExprKind::StaticMethodCall {
                receiver,
                method,
                args: self.hoist_exprs(args)?,
            },
            ExprKind::NewObject { class_name, args } => ExprKind::NewObject {
                class_name,
                args: self.hoist_exprs(args)?,
            },
            ExprKind::NewScopedObject { receiver, args } => ExprKind::NewScopedObject {
                receiver,
                args: self.hoist_exprs(args)?,
            },
            ExprKind::NewDynamic { name_expr, args } => ExprKind::NewDynamic {
                name_expr: self.hoist_boxed(name_expr)?,
                args: self.hoist_exprs(args)?,
            },
            ExprKind::NewDynamicObject {
                class_name,
                fallback_class,
                required_parent,
                args,
            } => ExprKind::NewDynamicObject {
                class_name: self.hoist_boxed(class_name)?,
                fallback_class,
                required_parent,
                args: self.hoist_exprs(args)?,
            },
            ExprKind::ExprCall { callee, args } => ExprKind::ExprCall {
                callee: self.hoist_boxed(callee)?,
                args: self.hoist_exprs(args)?,
            },
            ExprKind::ArrayLiteral(items) => ExprKind::ArrayLiteral(self.hoist_exprs(items)?),
            ExprKind::ArrayLiteralAssoc(entries) => {
                let mut out = Vec::with_capacity(entries.len());
                for (key, value) in entries {
                    let key = self.hoist_expr(key)?;
                    let value = self.hoist_expr(value)?;
                    out.push((key, value));
                }
                ExprKind::ArrayLiteralAssoc(out)
            }
            ExprKind::Match {
                subject,
                arms,
                default,
            } => {
                let subject = self.hoist_boxed(subject)?;
                let mut new_arms = Vec::with_capacity(arms.len());
                for (patterns, value) in arms {
                    let patterns = self.hoist_exprs(patterns)?;
                    let value = self.hoist_expr(value)?;
                    new_arms.push((patterns, value));
                }
                let default = match default {
                    Some(default) => Some(self.hoist_boxed(default)?),
                    None => None,
                };
                ExprKind::Match {
                    subject,
                    arms: new_arms,
                    default,
                }
            }
            ExprKind::PropertyAccess { object, property } => ExprKind::PropertyAccess {
                object: self.hoist_boxed(object)?,
                property,
            },
            ExprKind::NullsafePropertyAccess { object, property } => {
                ExprKind::NullsafePropertyAccess {
                    object: self.hoist_boxed(object)?,
                    property,
                }
            }
            ExprKind::DynamicPropertyAccess { object, property } => {
                ExprKind::DynamicPropertyAccess {
                    object: self.hoist_boxed(object)?,
                    property: self.hoist_boxed(property)?,
                }
            }
            ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
                ExprKind::NullsafeDynamicPropertyAccess {
                    object: self.hoist_boxed(object)?,
                    property: self.hoist_boxed(property)?,
                }
            }
            ExprKind::MethodCall {
                object,
                method,
                args,
            } => ExprKind::MethodCall {
                object: self.hoist_boxed(object)?,
                method,
                args: self.hoist_exprs(args)?,
            },
            ExprKind::NullsafeMethodCall {
                object,
                method,
                args,
            } => ExprKind::NullsafeMethodCall {
                object: self.hoist_boxed(object)?,
                method,
                args: self.hoist_exprs(args)?,
            },
            ExprKind::FirstClassCallable(target) => ExprKind::FirstClassCallable(match target {
                CallableTarget::Function(name) => CallableTarget::Function(name),
                CallableTarget::StaticMethod { receiver, method } => {
                    CallableTarget::StaticMethod { receiver, method }
                }
                CallableTarget::Method { object, method } => CallableTarget::Method {
                    object: self.hoist_boxed(object)?,
                    method,
                },
            }),
            // Closures resolve their body (`resolve_isolated`) and parameter defaults
            // (`resolve_params`) in isolation via `resolve_expr`, which re-enters this hoister at
            // the closure's own scope; hoisting here would run the include in the wrong scope.
            closure @ ExprKind::Closure { .. } => closure,
            // `$obj::CONST` — hoist any value-include inside the evaluated object sub-expression.
            ExprKind::DynamicClassConstantAccess { object, name } => {
                ExprKind::DynamicClassConstantAccess {
                    object: self.hoist_boxed(object)?,
                    name,
                }
            }
            // `self::${$expr}` — hoist any value-include inside the dynamic property-name expr.
            ExprKind::DynamicStaticPropertyAccess { receiver, property } => {
                ExprKind::DynamicStaticPropertyAccess {
                    receiver,
                    property: self.hoist_boxed(property)?,
                }
            }
            // Leaf expressions with no include-bearing children (mirrors the `false` group in
            // `contains::expr_has_includes`).
            leaf @ (ExprKind::StringLiteral(_)
            | ExprKind::IntLiteral(_)
            | ExprKind::FloatLiteral(_)
            | ExprKind::Variable(_)
            | ExprKind::BoolLiteral(_)
            | ExprKind::Null
            | ExprKind::PreIncrement(_)
            | ExprKind::PostIncrement(_)
            | ExprKind::PreDecrement(_)
            | ExprKind::PostDecrement(_)
            | ExprKind::ConstRef(_)
            | ExprKind::StaticPropertyAccess { .. }
            | ExprKind::This
            | ExprKind::ClassConstant { .. }
            | ExprKind::ScopedConstantAccess { .. }
            | ExprKind::MagicConstant(_)
            | ExprKind::Yield { .. }
            | ExprKind::YieldFrom(_)) => leaf,
        };
        Ok(Expr::new(kind, span))
    }

    /// Hoists value-includes inside a boxed expression, preserving the boxing.
    fn hoist_boxed(&mut self, expr: Box<Expr>) -> Result<Box<Expr>, CompileError> {
        Ok(Box::new(self.hoist_expr(*expr)?))
    }

    /// Hoists value-includes inside each expression of a vector, left-to-right (source order).
    fn hoist_exprs(&mut self, exprs: Vec<Expr>) -> Result<Vec<Expr>, CompileError> {
        let mut out = Vec::with_capacity(exprs.len());
        for expr in exprs {
            out.push(self.hoist_expr(expr)?);
        }
        Ok(out)
    }
}
