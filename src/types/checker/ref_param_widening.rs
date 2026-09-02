//! Purpose:
//! Program-wide pre-pass that records every DECLARED by-reference parameter whose own body stores
//! a value the declaration cannot represent, so the CALLER and the CALLEE agree on one boxed cell
//! before either body is walked.
//!
//! Called from:
//! - `crate::types::checker::driver::check_types_impl`, once, after the function and class
//!   declarations are collected and the builtin signatures are patched.
//!
//! Key details:
//! - The fact is inter-procedural by nature and cannot be discovered while checking a body. The
//!   callee's cell IS the caller's slot, so a write the callee makes only reaches the caller if
//!   the CALLER's slot can hold it — and the checker's walk order does not guarantee the callee is
//!   checked first (a top-level call is checked before `resolve_unchecked_functions` ever reaches
//!   the callee, and `type_check_methods_until_stable` iterates classes in map order).
//! - Purely a STORAGE decision. The declared type stays in the signature and keeps validating
//!   arguments at every call site, so `function adv(int &$i) {…}; $s = "x"; adv($s);` is still the
//!   `parameter $i expects Int, got Str` diagnostic it is today — which matters, because `php -n`
//!   throws a TypeError there and silently accepting it would be a wrong answer, not a lost note.
//! - Missing a widening costs the pre-existing `cannot reassign` error, never a wrong
//!   representation, so every rule here fails towards saying nothing.

use std::collections::HashMap;

use crate::names::php_symbol_key;
use crate::parser::ast::{Expr, ExprKind, Program, Stmt, StmtKind, TypeExpr};
use crate::span::Span;
use crate::types::traits::FlattenedClass;
use crate::types::PhpType;

use super::Checker;

impl Checker {
    /// Records the by-reference parameters whose bodies widen them, for every function and method
    /// in the program.
    ///
    /// PHP's reference cell is untyped: a declared `int &$i` gates what may be BOUND to it at the
    /// call, and nothing at all about what the body may later store through it. `php -n` runs
    /// `function adv(string $s, int &$i) { $i = strpos($s, ":", $i); }` and prints `int(2)` then
    /// `bool(false)` through the reference — `Symfony\Component\Yaml\Inline::parseMapping` is that
    /// exact shape, and `ProxyHelper::exportParameters` is its `?string &$args` twin, assigned an
    /// `array` by `explode`.
    ///
    /// elephc's cell is the caller's frame slot, so representing that needs BOTH sides to use a
    /// boxed one. The widened storage is published in two places, and they are the same statement
    /// read by two consumers:
    ///
    /// - [`Checker::widened_ref_params`], which the callee's environment seeding reads to bind the
    ///   parameter `mixed` for the body — so the incompatible store MERGES instead of reaching the
    ///   widening rule at all, exactly as a `mixed &$i` declaration already does today, and
    ///   `--strict-locals` accepts it for the same reason;
    /// - `by_ref_local_storage_types`, under the CALLEE's own scope key, which is the map
    ///   `LoweringContext::required_local_storage_type` consults when it declares the parameter's
    ///   slot. One entry gives the cell its boxed representation with no new plumbing, because the
    ///   scope keys the checker and EIR lowering use for a body are already the same string.
    ///
    /// The caller side is not recorded here: `prepare_by_ref_variable_storage` derives it per call
    /// site from the callee's effective storage, which is what makes an argument's own slot box.
    ///
    /// CLOSURES are deliberately out of scope. A closure's key is its SPAN inside an enclosing
    /// body (`nested_loop_storage_scope`), which nothing here can reconstruct without walking
    /// expressions in the same order the checker later does; a `function (&$x) { … }` that widens
    /// its parameter therefore keeps today's hard error rather than getting a decision the two
    /// sides might disagree about.
    pub(crate) fn scan_widened_ref_params(
        &mut self,
        program: &Program,
        classes: &[FlattenedClass],
    ) {
        let mut widened: Vec<(String, String)> = Vec::new();
        for stmt in program {
            let StmtKind::FunctionDecl { name, .. } = &stmt.kind else {
                continue;
            };
            let key = self
                .canonical_function_name_folded(name)
                .unwrap_or_else(|| name.clone());
            let Some(decl) = self.fn_decls.get(&key) else {
                continue;
            };
            let params: Vec<(String, Option<TypeExpr>, bool)> = decl
                .params
                .iter()
                .cloned()
                .zip(decl.param_types.iter().cloned())
                .zip(decl.ref_params.iter().copied())
                .map(|((name, type_expr), by_ref)| (name, type_expr, by_ref))
                .collect();
            self.collect_widened_ref_params(&key, &params, &decl.body.clone(), &mut widened);
        }
        for class in classes {
            for method in &class.methods {
                if !method.has_body {
                    continue;
                }
                let key = format!("{}::{}", class.name, method.name);
                let params: Vec<(String, Option<TypeExpr>, bool)> = method
                    .params
                    .iter()
                    .map(|(name, type_expr, _, by_ref)| {
                        (name.clone(), type_expr.clone(), *by_ref)
                    })
                    .collect();
                self.collect_widened_ref_params(&key, &params, &method.body, &mut widened);
            }
        }
        for (scope, param) in widened {
            // EIR lowering looks the callee's own parameter slot up by the body scope key it
            // builds from the DECLARATION, so that entry keeps the exact spelling. The checker's
            // set is keyed by the case-FOLDED spelling instead, because a call site names the
            // callee the way the CALL spells it and PHP function and method names are
            // case-insensitive.
            self.by_ref_local_storage_types
                .insert((scope.clone(), param.clone()), PhpType::Mixed);
            self.widened_ref_params
                .insert((folded_scope_key(&scope), param.clone()));
            self.widened_ref_param_decls.push((scope, param));
        }
    }

    /// Rewrites every widened by-reference parameter's type in the resolved signatures, once all
    /// checking is done.
    ///
    /// The signature is the ABI, and by this point it is nothing else: every diagnostic that reads
    /// it has already run, so the DECLARED type has finished doing the one job only it can do —
    /// rejecting an argument PHP rejects, `parameter $i expects Int, got Str`. What EIR lowering
    /// needs from the same field afterwards is the cell's runtime REPRESENTATION, and for these
    /// parameters that is a boxed `Mixed` on both sides of the call. Publishing it here rather
    /// than threading a parallel table is what keeps the callee's slot, the call site's argument
    /// coercion and `ir_lower::expr::ref_place_args`'s adapter decision reading ONE fact: the
    /// adapter exists to bridge a boxed caller slot to a CONCRETE declared cell, and a cell that
    /// is no longer concrete must not get one — measured, the concrete `int` temporary it built
    /// made `adv("ab:cd", $p)` print a raw pointer where `php -n` prints `int(2)`.
    ///
    /// Runs only on the success path: a program with errors never reaches lowering, and leaving
    /// the signatures alone keeps every diagnostic quoting the declared type.
    pub(crate) fn publish_widened_ref_param_signatures(&mut self) {
        for (scope, param) in std::mem::take(&mut self.widened_ref_param_decls) {
            let Some((owner, method_name)) = scope.split_once("::") else {
                if let Some(signature) = self.functions.get_mut(&scope) {
                    widen_signature_param(signature, &param);
                }
                continue;
            };
            let key = php_symbol_key(method_name);
            // EVERY class that runs this body, not just the one that declares it. `ClassInfo` is
            // flattened per class, so a subclass carries its own copy of an inherited method's
            // signature — and a call site reads the RECEIVER's copy. Leaving a descendant's copy
            // at the declared type would hand `ir_lower::expr::ref_place_args` a concrete
            // `expected` for a cell the callee opens boxed, which is the adapter shape that prints
            // a raw pointer. `method_impl_classes` is the same map codegen uses to find a method's
            // implementation, so "runs this body" cannot mean two different things in two places.
            let targets: Vec<String> = self
                .classes
                .iter()
                .filter(|(class_name, info)| {
                    let implementer = info
                        .method_impl_classes
                        .get(&key)
                        .or_else(|| info.static_method_impl_classes.get(&key));
                    match implementer {
                        Some(implementer) => implementer == owner,
                        // No entry means the method is not inherited from anywhere: the class
                        // runs the body only if it IS the owner.
                        None => class_name.as_str() == owner,
                    }
                })
                .map(|(class_name, _)| class_name.clone())
                .collect();
            for class_name in targets {
                let Some(info) = self.classes.get_mut(&class_name) else {
                    continue;
                };
                for signature in info
                    .methods
                    .get_mut(&key)
                    .into_iter()
                    .chain(info.static_methods.get_mut(&key))
                {
                    widen_signature_param(signature, &param);
                }
            }
        }
    }

    /// Records every declared by-reference parameter of ONE body that the body cannot keep at its
    /// declared representation.
    fn collect_widened_ref_params(
        &self,
        scope: &str,
        params: &[(String, Option<TypeExpr>, bool)],
        body: &[Stmt],
        widened: &mut Vec<(String, String)>,
    ) {
        let by_ref_declared: Vec<(&str, PhpType)> = params
            .iter()
            .filter(|(_, type_expr, by_ref)| *by_ref && type_expr.is_some())
            .filter_map(|(name, type_expr, _)| {
                let declared = self
                    .resolve_type_expr(type_expr.as_ref()?, Span::dummy())
                    .ok()?;
                Some((name.as_str(), declared))
            })
            .collect();
        if by_ref_declared.is_empty() {
            return;
        }
        let mut stores: HashMap<&str, Vec<&Expr>> = HashMap::new();
        for (name, _) in &by_ref_declared {
            stores.insert(name, Vec::new());
        }
        collect_local_stores(body, &mut stores);
        // No parameter is filtered out up front, including one already declared `mixed`: the
        // predicate below answers `false` for it (`mixed` merges everything and keeps its own
        // representation), and a declared UNION must NOT be filtered out on the strength of
        // having a boxed representation — `?string &$args` assigned `explode`'s array is exactly
        // `ProxyHelper::exportParameters`, whose cell is boxed already and whose BINDING the
        // checker still refuses.
        for (name, declared) in by_ref_declared {
            let widens = stores
                .get(name)
                .into_iter()
                .flatten()
                .filter_map(|value| self.stored_value_type(value))
                .any(|stored| self.store_leaves_declared_representation(&declared, &stored));
            if widens {
                widened.push((scope.to_string(), name.to_string()));
            }
        }
    }

    /// Returns whether storing `stored` into a `declared` cell takes it off the declaration's
    /// runtime REPRESENTATION.
    ///
    /// The merge is `Checker::merged_assignment_type`, the same one the assignment itself will
    /// run, so this cannot answer differently from the checker about what the name ends up
    /// holding. What is compared afterwards is not the type but the `codegen_repr`, because that
    /// is the only thing the cell has to be able to hold: a nullable `?string` merges `null` and
    /// `string` without leaving `Str` storage, an `int` merges `int` and stays, and both keep
    /// their raw cell. A merge that comes back `None` — `?string` against `explode`'s array — or
    /// one that lands on a different representation — `int` against `strpos`, whose registered
    /// return type is `mixed` — is what needs a boxed one.
    ///
    /// The acceptance predicates are deliberately NOT the test. `Checker::types_compatible` and
    /// `Checker::type_accepts` are gradual: they answer `true` for `mixed` against every scalar,
    /// which is right for a call boundary that will narrow at runtime and wrong for a reference
    /// cell that has to STORE the value.
    fn store_leaves_declared_representation(&self, declared: &PhpType, stored: &PhpType) -> bool {
        // An UNKNOWN value stored into a declared UNION. The union's cell is boxed already, so
        // this widens nothing at runtime — it only stops the body's binding rule from refusing a
        // store it cannot see the type of, which for a union `merged_assignment_type` does for
        // every value the union does not literally contain. `ProxyHelper::exportParameters`'
        // `?string &$args` is the shape: `$args = \explode(", ", $args, 2)` is
        // `cannot reassign $args from string|null to array<string>`, and the registered return
        // type of `explode` is the same coarse `mixed` every builtin whose real return type is a
        // union carries here (the precise one lives in a `check` hook that needs a call context
        // this pass has not got). A declared `mixed` is excluded because it already merges
        // everything, and a CONCRETE declaration is excluded because widening it really would
        // change the ABI — that needs the representation evidence below, not a shrug.
        if matches!(stored, PhpType::Mixed)
            && declared.codegen_repr() == PhpType::Mixed
            && !matches!(declared, PhpType::Mixed)
        {
            return true;
        }
        match self.merged_assignment_type(declared, stored) {
            None => true,
            // An UNKNOWN value stored into a declared CONTAINER is not evidence. `mixed` is what
            // every builtin whose real return type this pass cannot reach carries here, and for a
            // container that is nearly always the same kind of container coming back:
            // `function __elephc_usort_mixed(array &$values, …) { $values = \array_values($values); }`
            // — the compiler's own sort prelude — was boxed on the strength of `array_values`'
            // registered `mixed`, which turned an array cell into a `Mixed` one under callers and
            // a runtime that both expect a container pointer, and `usort` over objects segfaulted
            // (`codegen::arrays::callbacks::test_usort_objects_typed_comparator`).
            //
            // A raw SCALAR cell is the opposite case and keeps the evidence: it is a register or a
            // pair with no room for a tag, so an unknown value genuinely cannot live in it, and
            // that is `int &$i` against `\strpos` — the shape this whole pass exists for. The
            // asymmetry is about what the representation can HOLD, not about how sure the pass is.
            Some(merged) => {
                merged.codegen_repr() != declared.codegen_repr()
                    && (!matches!(stored, PhpType::Mixed) || declared_is_raw_scalar(declared))
            }
        }
    }

    /// Returns the type a body stores, or `None` when this pass cannot name it.
    ///
    /// Deliberately narrow: a value whose type is only known once the body is really checked is
    /// NOT evidence, because the cost of missing one is the pre-existing hard error while the cost
    /// of guessing one wrong is a boxed cell on both sides of every call. The shapes answered here
    /// are the ones whose type is a property of the callee's own declaration rather than of the
    /// surrounding flow — a literal, a cast, a concatenation, a container literal, a constructor,
    /// and a call whose signature is already registered (every builtin, and a user function or
    /// method that DECLARED its return type).
    fn stored_value_type(&self, value: &Expr) -> Option<PhpType> {
        match &value.kind {
            ExprKind::StringLiteral(_) => Some(PhpType::Str),
            ExprKind::IntLiteral(_) => Some(PhpType::Int),
            ExprKind::FloatLiteral(_) => Some(PhpType::Float),
            ExprKind::BoolLiteral(_) => Some(PhpType::Bool),
            ExprKind::Null => Some(PhpType::Void),
            ExprKind::ArrayLiteral(_) => Some(PhpType::Array(Box::new(PhpType::Mixed))),
            ExprKind::ArrayLiteralAssoc(_) => Some(PhpType::AssocArray {
                key: Box::new(PhpType::Mixed),
                value: Box::new(PhpType::Mixed),
            }),
            ExprKind::Cast { target, .. } => match target {
                crate::parser::ast::CastType::String => Some(PhpType::Str),
                crate::parser::ast::CastType::Int => Some(PhpType::Int),
                crate::parser::ast::CastType::Float => Some(PhpType::Float),
                crate::parser::ast::CastType::Bool => Some(PhpType::Bool),
                crate::parser::ast::CastType::Array => {
                    Some(PhpType::Array(Box::new(PhpType::Mixed)))
                }
                _ => None,
            },
            ExprKind::BinaryOp {
                op: crate::parser::ast::BinOp::Concat,
                ..
            } => Some(PhpType::Str),
            ExprKind::NewObject { class_name, .. } => {
                Some(PhpType::Object(class_name.as_str().to_string()))
            }
            ExprKind::Clone(inner) => self.stored_value_type(inner),
            ExprKind::ErrorSuppress(inner) => self.stored_value_type(inner),
            ExprKind::FunctionCall { name, .. } => {
                let key = php_symbol_key(name.trim_start_matches('\\'));
                if let Some(signature) = crate::types::builtin_call_sig(&key) {
                    return Some(signature.return_type);
                }
                let canonical = self
                    .canonical_function_name_folded(name)
                    .unwrap_or_else(|| name.to_string());
                // Only a DECLARED return type is a fact about the callee. An inferred one is a
                // fact about a walk that has not happened yet.
                let declaration = self.fn_decls.get(&canonical)?;
                let return_type = declaration.return_type.clone()?;
                self.resolve_type_expr(&return_type, value.span).ok()
            }
            _ => None,
        }
    }
}

/// Collects every statement-form assignment `$name = <expr>` for the tracked names, at any
/// conditional depth.
///
/// Depth is irrelevant to this question: a widening keeps the slot and boxes it, so it is sound on
/// the taken and the untaken path alike — the same reason `Checker::local_binding_is_widenable`
/// drops the conditional-depth condition the kill needs. Closure bodies are not descended into
/// (see [`Checker::scan_widened_ref_params`]), and neither are nested function or class
/// declarations, whose parameters are their own scope's.
fn collect_local_stores<'a>(stmts: &'a [Stmt], out: &mut HashMap<&str, Vec<&'a Expr>>) {
    for stmt in stmts {
        collect_local_stores_in_stmt(stmt, out);
    }
}

/// Walks one statement and the executable bodies nested inside it.
fn collect_local_stores_in_stmt<'a>(stmt: &'a Stmt, out: &mut HashMap<&str, Vec<&'a Expr>>) {
    match &stmt.kind {
        StmtKind::Assign { name, value } | StmtKind::TypedAssign { name, value, .. } => {
            if let Some(values) = out.get_mut(name.as_str()) {
                values.push(value);
            }
            collect_local_stores_in_expr(value, out);
        }
        StmtKind::ExprStmt(expr) | StmtKind::Echo(expr) | StmtKind::Throw(expr) => {
            collect_local_stores_in_expr(expr, out)
        }
        StmtKind::Return(Some(expr)) => collect_local_stores_in_expr(expr, out),
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            collect_local_stores_in_expr(condition, out);
            collect_local_stores(then_body, out);
            for (clause_condition, body) in elseif_clauses {
                collect_local_stores_in_expr(clause_condition, out);
                collect_local_stores(body, out);
            }
            if let Some(body) = else_body {
                collect_local_stores(body, out);
            }
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            collect_local_stores_in_expr(condition, out);
            collect_local_stores(body, out);
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            if let Some(init) = init {
                collect_local_stores_in_stmt(init, out);
            }
            if let Some(condition) = condition {
                collect_local_stores_in_expr(condition, out);
            }
            if let Some(update) = update {
                collect_local_stores_in_stmt(update, out);
            }
            collect_local_stores(body, out);
        }
        StmtKind::Foreach { array, body, .. } => {
            collect_local_stores_in_expr(array, out);
            collect_local_stores(body, out);
        }
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            collect_local_stores_in_expr(subject, out);
            for (values, body) in cases {
                for value in values {
                    collect_local_stores_in_expr(value, out);
                }
                collect_local_stores(body, out);
            }
            if let Some(body) = default {
                collect_local_stores(body, out);
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            collect_local_stores(try_body, out);
            for catch in catches {
                collect_local_stores(&catch.body, out);
            }
            if let Some(body) = finally_body {
                collect_local_stores(body, out);
            }
        }
        StmtKind::Synthetic(body)
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. } => collect_local_stores(body, out),
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            collect_local_stores(then_body, out);
            if let Some(body) = else_body {
                collect_local_stores(body, out);
            }
        }
        _ => {}
    }
}

/// Walks the statement lists an expression carries, for the assignment forms that live there.
///
/// An `$x = …` written in EXPRESSION position (`$a = $x = f();`, `while ($x = next())`) is an
/// `ExprKind::Assignment`, not a `StmtKind::Assign`, and it stores through the reference just the
/// same.
fn collect_local_stores_in_expr<'a>(expr: &'a Expr, out: &mut HashMap<&str, Vec<&'a Expr>>) {
    match &expr.kind {
        ExprKind::Assignment {
            target,
            value,
            prelude,
            ..
        } => {
            collect_local_stores(prelude, out);
            if let ExprKind::Variable(name) = &target.kind {
                if let Some(values) = out.get_mut(name.as_str()) {
                    values.push(value);
                }
            }
            collect_local_stores_in_expr(value, out);
        }
        ExprKind::BinaryOp { left, right, .. } => {
            collect_local_stores_in_expr(left, out);
            collect_local_stores_in_expr(right, out);
        }
        ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Print(inner)
        | ExprKind::Cast { expr: inner, .. } => collect_local_stores_in_expr(inner, out),
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            collect_local_stores_in_expr(condition, out);
            collect_local_stores_in_expr(then_expr, out);
            collect_local_stores_in_expr(else_expr, out);
        }
        ExprKind::FunctionCall { args, .. } => {
            for arg in args {
                collect_local_stores_in_expr(arg, out);
            }
        }
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            collect_local_stores_in_expr(object, out);
            for arg in args {
                collect_local_stores_in_expr(arg, out);
            }
        }
        ExprKind::StaticMethodCall { args, .. } => {
            for arg in args {
                collect_local_stores_in_expr(arg, out);
            }
        }
        _ => {}
    }
}

impl Checker {
    /// Returns the type a body's environment binds `param` to on entry.
    ///
    /// `PhpType::Mixed` for a by-reference parameter [`Checker::scan_widened_ref_params`] widened,
    /// and `declared` unchanged for everything else. Applied to the ENVIRONMENT only: the
    /// signature keeps the declared type, so a call site still reports
    /// `parameter $i expects Int, got Str` for an argument PHP rejects with a TypeError.
    pub(crate) fn widened_ref_param_env_type(
        &self,
        scope: &str,
        param: &str,
        declared: &PhpType,
    ) -> PhpType {
        if self.ref_param_is_widened(scope, param) {
            PhpType::Mixed
        } else {
            declared.clone()
        }
    }

    /// Returns whether `param`'s by-reference cell in `scope` was widened to boxed storage.
    pub(crate) fn ref_param_is_widened(&self, scope: &str, param: &str) -> bool {
        self.widened_ref_params
            .contains(&(folded_scope_key(scope), param.to_string()))
    }

    /// Returns the storage a CALL SITE must give the caller's variable for one by-reference
    /// parameter of `callee`.
    ///
    /// The declared type, except where the callee's body widens the parameter — there the cell is
    /// boxed and the caller's slot has to be boxed with it, or the callee's write lands in a slot
    /// that cannot hold it and the caller reads the box back as whatever its declared
    /// representation says. `None` for a callee this pass cannot name (a closure, a runtime
    /// callable, a `$f(...)`), which leaves the call on its pre-existing storage decision.
    pub(crate) fn effective_ref_param_type(
        &self,
        callee_scope: Option<&str>,
        param: &str,
        declared: &PhpType,
    ) -> PhpType {
        match callee_scope {
            Some(scope) => self.widened_ref_param_env_type(scope, param, declared),
            None => declared.clone(),
        }
    }

    /// Names the BODY that a `Class::method` call actually runs, for the widening lookup.
    ///
    /// An inherited method's body belongs to the class that DECLARED it, and that is the scope the
    /// pre-pass scanned — `$child->m($x)` where `m` comes from `Parent` has to ask about
    /// `Parent::m`. `method_impl_classes` is the same map codegen uses to find a method's
    /// implementation, so the two cannot drift; a method with no entry is answered against the
    /// receiver's own class, which is the non-inherited case.
    pub(crate) fn method_body_scope(&self, class_name: &str, method: &str) -> Option<String> {
        let class_name = class_name.trim_start_matches('\\');
        let info = self.classes.get(class_name)?;
        let key = php_symbol_key(method);
        let owner = info
            .method_impl_classes
            .get(&key)
            .or_else(|| info.static_method_impl_classes.get(&key))
            .map(String::as_str)
            .unwrap_or(class_name);
        Some(format!("{}::{}", owner, method))
    }
}

/// Case-folds a body scope key so a call site's spelling of the callee matches the declaration's.
///
/// PHP function and class/method names are case-insensitive, and the scope key is built from the
/// DECLARATION on one side and from the CALL on the other. `php_symbol_key` is applied per segment
/// so `Foo::Bar` and `foo::bar` agree without `Foo::bar` colliding with a function named `foo::bar`
/// (which cannot exist).
fn folded_scope_key(scope: &str) -> String {
    match scope.split_once("::") {
        Some((class, method)) => {
            format!("{}::{}", php_symbol_key(class), php_symbol_key(method))
        }
        None => php_symbol_key(scope),
    }
}

/// Replaces one named parameter's type with boxed `Mixed` in a resolved signature.
fn widen_signature_param(signature: &mut crate::types::FunctionSig, param: &str) {
    for (name, ty) in &mut signature.params {
        if name == param {
            *ty = PhpType::Mixed;
        }
    }
}

/// Returns whether a declared type's cell is a raw scalar — a register or a register pair with no
/// room for a runtime tag.
///
/// `TaggedScalar` is included: a nullable integer carries its tag in a second word rather than in a
/// box, so it can no more hold an arbitrary value than a bare `int` can.
fn declared_is_raw_scalar(declared: &PhpType) -> bool {
    matches!(
        declared.codegen_repr(),
        PhpType::Int
            | PhpType::Float
            | PhpType::Bool
            | PhpType::False
            | PhpType::Str
            | PhpType::TaggedScalar
    )
}
