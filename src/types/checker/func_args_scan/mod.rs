//! Purpose:
//! Detects user functions/methods whose body calls `func_num_args()`, `func_get_args()`,
//! or `func_get_arg()`, and validates that no such call appears at the top-level (global)
//! scope, mirroring PHP's own "cannot be called from the global scope" fatal.
//!
//! Called from:
//! - `crate::types::checker::driver::mod` — runs `mark_func_args_functions` after all
//!   user function/method signatures are resolved, and `validate_func_args_global_scope`
//!   alongside the other program-wide validation passes.
//! - `crate::ir_lower` — consumes `CheckResult::func_args_functions` to thread the hidden
//!   arity-count/variadic-tail ABI extension through call sites and callee prologues.
//!
//! Key details:
//! - "Closed-world": only DIRECT free-function calls and methods with an unambiguous
//!   implementation get the hidden ABI extension. Closures are intentionally out of scope (see
//!   `detect::body_calls_func_args_intrinsic`, which stops at `Closure` boundaries) and
//!   are rejected defensively in `crate::ir_lower` if reached.
//! - ...EXCEPT when marking the method changes nothing about its ABI: see
//!   `relaxation_is_abi_neutral`. A body whose declared parameters are ALL required computes
//!   `func_num_args()` from its own frame (`argc_is_self_derivable`), so it needs no hidden
//!   operand; if it ALSO already declares a variadic tail, its lowered signature is identical
//!   to what it would be if the body never mentioned `func_get_args()`, and the closed-world
//!   gate has nothing left to protect.
//! - `mark_func_args_functions` mutates each matched signature's `variadic` slot in place
//!   (synthesizing one when the function was not already variadic) so 100% of the existing
//!   variadic call-argument-count relaxation, named-argument exclusion, and EIR local/ABI
//!   slot materialization machinery is reused instead of re-implemented.

mod detect;

pub(crate) use detect::{
    body_calls_func_args_intrinsic, expr_calls_func_args_intrinsic, is_func_args_intrinsic_name,
};

use std::collections::HashSet;

use crate::errors::CompileError;
use crate::parser::ast::{Program, StmtKind};
use crate::types::FunctionSig;

/// Synthetic trailing variadic parameter name appended to a function/method signature that
/// uses `func_num_args()`/`func_get_args()`/`func_get_arg()` but was not already declared
/// variadic. Collects positional arguments beyond the declared parameters, exactly like a
/// real `...$rest` parameter, so `func_get_args()` can read them back.
pub(crate) const SYNTHETIC_VARIADIC_NAME: &str = "__fga_variadic";

/// Hidden trailing arity-count parameter name. Carries the number of PHP-visible
/// positional/named-resolved arguments the caller actually passed (PHP's
/// `func_num_args()` semantics: highest-filled parameter index + 1, plus any variadic
/// tail count). Threaded as a plain EIR ABI parameter OUTSIDE the checker-visible
/// `FunctionSig::params` list (see `crate::ir_lower::function`), so caller-visible
/// arg-count/named-argument matching never sees it.
pub(crate) const HIDDEN_ARGC_PARAM_NAME: &str = "__fga_argc";

/// Ensures `sig` accepts unlimited trailing positional arguments by synthesizing a
/// `SYNTHETIC_VARIADIC_NAME` variadic slot when `sig` is not already variadic. Reuses the
/// existing variadic parameter untouched when one was already declared (e.g.
/// `function f($a, ...$rest)` that also calls `func_get_args()`).
pub(crate) fn ensure_variadic_for_func_args(sig: &mut FunctionSig) {
    if sig.variadic.is_some() {
        return;
    }
    sig.params.push((
        SYNTHETIC_VARIADIC_NAME.to_string(),
        crate::types::PhpType::Array(Box::new(crate::types::PhpType::Mixed)),
    ));
    // Matches `crate::types::signatures::variadic()`'s convention: the variadic slot's
    // own `defaults` entry is `Some(<empty array literal>)`, not `None` — arg-count
    // validation (`call_validation::required`) counts `None` defaults as "required
    // parameters", so a bare `None` here would wrongly demand a value for the variadic
    // tail itself.
    sig.defaults.push(Some(crate::parser::ast::Expr::new(
        crate::parser::ast::ExprKind::ArrayLiteral(Vec::new()),
        crate::span::Span::dummy(),
    )));
    sig.ref_params.push(false);
    sig.declared_params.push(false);
    sig.variadic = Some(SYNTHETIC_VARIADIC_NAME.to_string());
}

/// Returns `true` when `func_num_args()` is derivable INSIDE the body from its own frame,
/// with no cooperation from any caller: every declared regular parameter is required, so it
/// was necessarily passed, and the argument count is exactly
/// `regular_param_count + count($<variadic tail>)`.
///
/// This is precisely the condition under which the hidden `HIDDEN_ARGC_PARAM_NAME` ABI
/// operand is unnecessary — and with it every restriction that operand imposes. A callee that
/// needs nothing from its callers can be reached through ANY dispatch shape, so
/// `method_has_one_closed_world_implementation` and the dynamic-length-spread rejection are
/// both skipped for it. `crate::ir_lower::expr::func_args_intrinsics` reads the same predicate
/// to decide between `$__fga_argc` and the self-derived count; keeping ONE predicate (rather
/// than a second marked set threaded through lowering) is what keeps the two sides in
/// lockstep, and a divergence would surface as a loud EIR operand/parameter count mismatch
/// rather than a silent miscompile.
///
/// A DEFAULT is exactly what breaks this: `f(int $max = 0)` cannot tell `f()` from `f(0)` by
/// looking at `$max`, yet PHP reports 0 vs 1 (php -n verified) — those bodies keep the hidden
/// operand and the closed-world gate.
///
/// The predicate is invariant to whether `ensure_variadic_for_func_args` has already run:
/// `regular_param_count` excludes the variadic slot, so the set of parameters inspected is the
/// declared one either way.
pub(crate) fn argc_is_self_derivable(sig: &FunctionSig) -> bool {
    (0..crate::types::call_args::regular_param_count(sig))
        .all(|index| sig.defaults.get(index).map_or(true, Option::is_none))
}

/// Returns `true` when marking `sig` arity-hungry changes its ABI in NO way at all: it needs
/// no hidden argc operand (`argc_is_self_derivable`), and `ensure_variadic_for_func_args` will
/// add nothing because the source already declares a real variadic tail.
///
/// This — not `argc_is_self_derivable` alone — is the condition under which
/// `method_has_one_closed_world_implementation` can be skipped. The gate protects TWO things,
/// and the synthetic variadic is the one that is easy to miss: giving `C::m($a)` a synthesized
/// tail while an override `D::m($a)` keeps two parameters makes the two implementations
/// disagree on their operand layout, so a union/`Mixed` receiver dispatching by name reaches
/// one of them with the wrong frame (measured: `$value->m(1)` on `true ? new C() : new D()`
/// compiles and then dies with an uncaught exception). A signature that ALREADY declares
/// `...$rest` is immune: its lowered form is byte-identical to what it would be if the body
/// never mentioned `func_get_args()`, so nothing about dispatch can change.
///
/// The synthetic name is excluded explicitly because `mark_func_args_functions` runs twice
/// (see `crate::types::checker::driver`), and the second pass must not mistake the first
/// pass's own synthesized tail for a source-declared one.
pub(crate) fn relaxation_is_abi_neutral(sig: &FunctionSig) -> bool {
    sig.variadic
        .as_deref()
        .is_some_and(|name| name != SYNTHETIC_VARIADIC_NAME)
        && argc_is_self_derivable(sig)
}

/// Looks up the resolved signature of `class_name::method_key`, honoring the static/instance
/// split, for `argc_is_self_derivable` queries.
pub(crate) fn method_signature<'a>(
    classes: &'a std::collections::HashMap<String, crate::types::ClassInfo>,
    class_name: &str,
    method_key: &str,
    is_static: bool,
) -> Option<&'a FunctionSig> {
    let class_info = classes.get(class_name)?;
    if is_static {
        class_info.static_methods.get(method_key)
    } else {
        class_info.methods.get(method_key)
    }
}

/// Scans every top-level (non-declaration) statement of the program for a call to
/// `func_num_args()`, `func_get_args()`, or `func_get_arg()` at the global scope and
/// returns one `CompileError` per illegal call site, mirroring PHP's own
/// `"func_get_args() cannot be called from the global scope"` runtime fatal as a
/// compile-time diagnostic instead.
pub(crate) fn validate_func_args_global_scope(program: &Program) -> Vec<CompileError> {
    let mut errors = Vec::new();
    let top_level: Vec<_> = program
        .iter()
        .filter(|stmt| {
            !matches!(
                stmt.kind,
                StmtKind::FunctionDecl { .. }
                    | StmtKind::ClassDecl { .. }
                    | StmtKind::TraitDecl { .. }
                    | StmtKind::InterfaceDecl { .. }
                    | StmtKind::EnumDecl { .. }
                    | StmtKind::FunctionVariantGroup { .. }
            )
        })
        .cloned()
        .collect();
    if body_calls_func_args_intrinsic(&top_level) {
        for stmt in &top_level {
            collect_global_scope_error(stmt, &mut errors);
        }
    }
    errors
}

/// Recurses into `stmt` reusing the same scope-stopping rules as
/// `detect::body_calls_func_args_intrinsic`, emitting one `CompileError` per direct
/// `func_num_args`/`func_get_args`/`func_get_arg` call found at global scope.
fn collect_global_scope_error(stmt: &crate::parser::ast::Stmt, errors: &mut Vec<CompileError>) {
    use crate::parser::ast::ExprKind;
    match &stmt.kind {
        StmtKind::ExprStmt(expr) | StmtKind::Echo(expr) | StmtKind::Throw(expr) => {
            collect_global_scope_expr_error(expr, errors)
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            collect_global_scope_expr_error(condition, errors);
            for s in then_body {
                collect_global_scope_error(s, errors);
            }
            for (c, body) in elseif_clauses {
                collect_global_scope_expr_error(c, errors);
                for s in body {
                    collect_global_scope_error(s, errors);
                }
            }
            if let Some(body) = else_body {
                for s in body {
                    collect_global_scope_error(s, errors);
                }
            }
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            collect_global_scope_expr_error(condition, errors);
            for s in body {
                collect_global_scope_error(s, errors);
            }
        }
        StmtKind::For {
            condition, body, ..
        } => {
            if let Some(condition) = condition {
                collect_global_scope_expr_error(condition, errors);
            }
            for s in body {
                collect_global_scope_error(s, errors);
            }
        }
        StmtKind::Foreach { array, body, .. } => {
            collect_global_scope_expr_error(array, errors);
            for s in body {
                collect_global_scope_error(s, errors);
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            for s in try_body {
                collect_global_scope_error(s, errors);
            }
            for c in catches {
                for s in &c.body {
                    collect_global_scope_error(s, errors);
                }
            }
            if let Some(finally_body) = finally_body {
                for s in finally_body {
                    collect_global_scope_error(s, errors);
                }
            }
        }
        StmtKind::Synthetic(stmts) | StmtKind::NamespaceBlock { body: stmts, .. } => {
            for s in stmts {
                collect_global_scope_error(s, errors);
            }
        }
        StmtKind::Assign { value, .. }
        | StmtKind::TypedAssign { value, .. }
        | StmtKind::ConstDecl { value, .. } => collect_global_scope_expr_error(value, errors),
        StmtKind::Return(Some(expr)) => collect_global_scope_expr_error(expr, errors),
        _ => {
            let _ = ExprKind::Null; // keep the `use` above meaningful without a catch-all match arm.
        }
    }
}

/// Emits a `CompileError` at `expr`'s span if it directly calls a
/// `func_num_args`/`func_get_args`/`func_get_arg` intrinsic (not skipping into closures),
/// then recurses to catch further illegal calls in sub-expressions.
fn collect_global_scope_expr_error(expr: &crate::parser::ast::Expr, errors: &mut Vec<CompileError>) {
    use crate::parser::ast::ExprKind;
    if let ExprKind::FunctionCall { name, args } = &expr.kind {
        if detect::is_func_args_intrinsic_name(name.as_str()) {
            errors.push(CompileError::new(
                expr.span,
                &format!(
                    "{}(): Cannot call {}() from the global scope",
                    name.as_str(),
                    name.as_str()
                ),
            ));
        }
        for arg in args {
            collect_global_scope_expr_error(arg, errors);
        }
        return;
    }
    if expr_calls_func_args_intrinsic(expr) {
        // A nested (non-FunctionCall) expression still routes through the shared
        // detector for coverage; report at the outer expression's span since the
        // exact inner call site does not need per-arg granularity here.
        errors.push(CompileError::new(
            expr.span,
            "func_num_args()/func_get_args()/func_get_arg() cannot be called from the global scope",
        ));
    }
}

/// Returns `true` when `args` contains a spread (`...`) argument whose length cannot be
/// determined at compile time. Mirrors `crate::ir_lower::expr`'s
/// `has_static_call_spread_args`/`expand_static_call_spread_args`: a spread of an array OR
/// associative-array LITERAL is statically flattenable (its element count is just the
/// literal's length), so only a spread of anything else (a variable, a function call
/// result, ...) counts as "dynamic" here.
pub(crate) fn call_has_dynamic_spread(args: &[crate::parser::ast::Expr]) -> bool {
    use crate::parser::ast::ExprKind;
    args.iter().any(|arg| match &arg.kind {
        ExprKind::Spread(inner) => !matches!(
            inner.kind,
            ExprKind::ArrayLiteral(_) | ExprKind::ArrayLiteralAssoc(_)
        ),
        _ => false,
    })
}

/// Returns the marked `"Class::method"` key of the arity-hungry implementation named
/// `method` when exactly ONE marked key carries that name — the checker-side mirror of
/// `crate::ir_lower::expr`'s `instance_method_arity_hungry_callee_key` name-only fallback,
/// including its "give up when ambiguous" rule.
///
/// For an ordinary method the lookup is EXACT rather than merely conservative:
/// `method_has_one_closed_world_implementation` only marks a method when its name resolves
/// to a single implementation program-wide, so at most one marked key can ever end with
/// `"::<method_key>"` and no unmarked class shares the name with a different ABI.
/// Constructors are marked WITHOUT that gate, so several `"::__construct"` keys can be
/// marked at once; returning `None` then is not a hole but the same answer lowering gives —
/// it appends no hidden operand for an ambiguous name either, so no `panic!` is reachable.
/// (Measured: `ContainerBuilder.php:1195`'s `$tryProxy->__construct(...$arguments)` on a
/// gradual receiver is exactly this case, and rejecting it would be a false positive.)
pub(crate) fn marked_method_key_for_name(marked: &HashSet<String>, method: &str) -> Option<String> {
    let suffix = format!("::{}", crate::names::php_symbol_key(method));
    let mut candidates = marked
        .iter()
        .filter(|candidate| candidate.ends_with(&suffix));
    let candidate = candidates.next()?.clone();
    candidates.next().is_none().then_some(candidate)
}

/// Returns the marked `"Class::method"` key a SINGULAR `class_name` receiver would dispatch
/// to, mirroring `crate::ir_lower::expr`'s `instance_method_arity_hungry_callee_key` singular
/// branch (which resolves the implementation through `ClassInfo::method_impl_classes` before
/// consulting the marked set). Needed alongside `marked_method_key_for_name` because that
/// name-only rule abstains on an ambiguous name, while lowering still resolves — and appends
/// the hidden operand for — a statically typed receiver.
pub(crate) fn marked_method_key_for_receiver_class(
    marked: &HashSet<String>,
    classes: &std::collections::HashMap<String, crate::types::ClassInfo>,
    class_name: &str,
    method: &str,
) -> Option<String> {
    let method_key = crate::names::php_symbol_key(method);
    let class_info = classes.get(class_name)?;
    let implementation = class_info
        .method_impl_classes
        .get(&method_key)
        .map(String::as_str)
        .unwrap_or(class_name);
    let key = format!("{}::{}", implementation, method_key);
    marked.contains(&key).then_some(key)
}

/// Builds the `CompileError` for a call into an arity-hungry function/method with a
/// dynamic-length spread argument (see `call_has_dynamic_spread`) — the caller-visible
/// counterpart of `crate::ir_lower::expr::func_args_intrinsics`'s defense-in-depth panic.
pub(crate) fn dynamic_spread_call_error(
    callee_desc: &str,
    span: crate::span::Span,
) -> CompileError {
    CompileError::new(
        span,
        &format!(
            "{} calls func_num_args()/func_get_args()/func_get_arg() and cannot be called \
             with a dynamic-length spread (`...`) argument — its element count is not known \
             at compile time; call it with explicit positional/named arguments, or spread a \
             literal array (e.g. `f(...[1, 2, 3])`) instead",
            callee_desc
        ),
    )
}

/// Scans every class method body for an unsupported call to `func_num_args`/
/// `func_get_args`/`func_get_arg` and returns one `CompileError` per rejected method.
///
/// `marked` contains methods whose closed-world implementation is unambiguous and can
/// therefore receive the hidden argc operand safely. Methods outside that set stay loud:
/// virtual dispatch could otherwise select an override with a different ABI.
pub(crate) fn validate_func_args_method_bodies(
    class_method_bodies: &std::collections::HashMap<String, Vec<(String, bool, Vec<crate::parser::ast::Stmt>)>>,
    marked: &HashSet<String>,
) -> Vec<CompileError> {
    let mut errors = Vec::new();
    for (class_name, methods) in class_method_bodies {
        for (method_name, _is_static, body) in methods {
            if !body_calls_func_args_intrinsic(body) {
                continue;
            }
            let method_key = format!(
                "{}::{}",
                class_name,
                crate::names::php_symbol_key(method_name)
            );
            if marked.contains(&method_key) {
                continue;
            }
            let span = first_func_args_intrinsic_span(body).unwrap_or(crate::span::Span::dummy());
            errors.push(CompileError::new(
                span,
                &format!(
                    "Method {}::{}() calls func_num_args()/func_get_args()/func_get_arg(), \
                     which this compiler does not support in methods — a method call site's \
                     receiver type may not determine which concrete (possibly overriding) \
                     implementation runs, so the caller cannot always supply the required \
                     hidden argument-count information. Extract the logic into a free \
                     function instead.",
                    class_name, method_name
                ),
            ));
        }
    }
    errors
}

/// Finds the span of the first `func_num_args`/`func_get_args`/`func_get_arg` call in
/// `body`, for error reporting. Reuses the same scope-stopping traversal as
/// `detect::body_calls_func_args_intrinsic` by scanning statement-by-statement until one
/// reports a hit.
fn first_func_args_intrinsic_span(body: &[crate::parser::ast::Stmt]) -> Option<crate::span::Span> {
    body.iter()
        .find(|stmt| body_calls_func_args_intrinsic(std::slice::from_ref(stmt)))
        .map(|stmt| stmt.span)
}

/// Computes the set of user function/method canonical keys whose body calls
/// `func_num_args`/`func_get_args`/`func_get_arg` at its own scope, mutating each such
/// signature via `ensure_variadic_for_func_args`. Free functions are keyed by their
/// canonical name (matching `CheckResult::functions`). Methods are keyed as
/// `"ClassName::lowercase_method_key"`.
///
/// A non-constructor method whose relaxation is NOT ABI-neutral (`relaxation_is_abi_neutral`)
/// is marked only when every class exposing that method name
/// dispatches to the same implementation. This deliberately strong closed-world condition
/// keeps gradual/interface receiver calls safe too: one call shape can never select a
/// non-arity-hungry unrelated implementation at runtime. Constructors are direct allocation
/// targets and are therefore marked independently; ordinary singular receiver calls to
/// `__construct` still resolve their concrete implementation before appending argc.
pub(crate) fn mark_func_args_functions(
    functions: &mut std::collections::HashMap<String, FunctionSig>,
    fn_decl_bodies: &std::collections::HashMap<String, Vec<crate::parser::ast::Stmt>>,
    classes: &mut std::collections::HashMap<String, crate::types::ClassInfo>,
    class_method_bodies: &std::collections::HashMap<String, Vec<(String, bool, Vec<crate::parser::ast::Stmt>)>>,
) -> HashSet<String> {
    let mut marked = HashSet::new();
    for (name, body) in fn_decl_bodies {
        if !body_calls_func_args_intrinsic(body) {
            continue;
        }
        let Some(sig) = functions.get_mut(name) else {
            continue;
        };
        ensure_variadic_for_func_args(sig);
        marked.insert(name.clone());
    }
    let mut method_candidates = Vec::new();
    for (class_name, methods) in class_method_bodies {
        for (method_name, is_static, body) in methods {
            if body_calls_func_args_intrinsic(body) {
                method_candidates.push((
                    class_name.clone(),
                    crate::names::php_symbol_key(method_name),
                    *is_static,
                ));
            }
        }
    }
    for (class_name, method_key, is_static) in method_candidates {
        // An ABI-NEUTRAL relaxation leaves the lowered signature exactly as declared — no
        // hidden operand, no synthesized tail — so no dispatch shape can get its operand layout
        // wrong and the closed-world gate below has nothing left to protect. This is what lets
        // `Dotenv::load(string $path, string ...$extraPaths)` compile: `load` has dozens of
        // unrelated implementations in Symfony, which the gate cannot distinguish from a
        // genuinely ambiguous ABI. `ArrayAdapter::getValues()` deliberately does NOT qualify —
        // it declares no variadic, so relaxing it would ADD a parameter that any other
        // `getValues()` implementation reached by name dispatch would not have.
        let abi_neutral = method_signature(classes, &class_name, &method_key, is_static)
            .is_some_and(relaxation_is_abi_neutral);
        if !abi_neutral
            && method_key != "__construct"
            && !method_has_one_closed_world_implementation(
                classes,
                &class_name,
                &method_key,
                is_static,
            )
        {
            continue;
        }
        relax_method_implementation_signatures(classes, &class_name, &method_key, is_static);
        marked.insert(format!("{}::{}", class_name, method_key));
    }
    marked
}

/// Returns whether every class exposing `method_key` dispatches to `owner`.
///
/// This is intentionally global rather than descendant-only: a `Mixed`/interface receiver
/// dispatches by method NAME in `crate::ir_lower::expr`'s
/// `instance_method_arity_hungry_callee_key` fallback, so mixing one hidden-argc
/// implementation with one ordinary implementation would make its call operand layout
/// ambiguous. That name-only fallback is a lockstep invariant of this predicate: relaxing
/// the rule to a descendant-only one without changing the fallback would silently append
/// (or omit) the hidden argc operand on a dynamically dispatched call.
///
/// A class that merely *declares* `method_key` without a body — an interface member, or an
/// abstract class that inherited an interface/abstract declaration it does not implement —
/// is NOT an implementation and is skipped. `ClassInfo::methods` carries such declarations
/// (see `crate::types::checker::schema::classes::methods`, which removes the
/// `method_impl_classes` entry for an abstract method while keeping the signature), and
/// counting them as distinct implementations used to reject the extremely common
/// `interface I { m(); } abstract class B implements I {} class C extends B { m() {...} }`
/// shape even though `C::m` is the only body dispatch can ever reach.
fn method_has_one_closed_world_implementation(
    classes: &std::collections::HashMap<String, crate::types::ClassInfo>,
    owner: &str,
    method_key: &str,
    is_static: bool,
) -> bool {
    let mut implementations = HashSet::new();
    for class_info in classes.values() {
        let (signatures, implementation_classes) = if is_static {
            (
                &class_info.static_methods,
                &class_info.static_method_impl_classes,
            )
        } else {
            (&class_info.methods, &class_info.method_impl_classes)
        };
        if !signatures.contains_key(method_key) {
            continue;
        }
        let Some(implementation) = implementation_classes.get(method_key) else {
            continue;
        };
        implementations.insert(implementation.clone());
    }
    implementations.len() == 1 && implementations.contains(owner)
}

/// Adds the synthetic variadic tail to the owning method signature and every inherited
/// signature copy that still dispatches to that implementation.
///
/// Deliberately does NOT relax the bodyless DECLARATION copies (an interface member, or an
/// abstract class that inherited a declaration it does not implement), even though the
/// closed-world gate has proven every dispatch reaches `owner`. Measured: relaxing them makes
/// the checker accept `$abstractTyped->m($declared, $extra)` while codegen still sizes the
/// call's operand list from the RECEIVER-typed declaration, producing
/// `unsupported EIR backend feature: method call to Base::m with 4 operands for 3 ABI params`
/// — a checker/backend split. Supporting an abstract- or interface-typed receiver needs the
/// hidden argc threaded through the declaration's ABI as well, not just its arity.
fn relax_method_implementation_signatures(
    classes: &mut std::collections::HashMap<String, crate::types::ClassInfo>,
    owner: &str,
    method_key: &str,
    is_static: bool,
) {
    let affected_classes: Vec<String> = classes
        .iter()
        .filter_map(|(class_name, class_info)| {
            let implementation_classes = if is_static {
                &class_info.static_method_impl_classes
            } else {
                &class_info.method_impl_classes
            };
            let implementation = implementation_classes
                .get(method_key)
                .map(String::as_str)
                .unwrap_or(class_name);
            (implementation == owner).then(|| class_name.clone())
        })
        .collect();
    for class_name in affected_classes {
        let Some(class_info) = classes.get_mut(&class_name) else {
            continue;
        };
        let signatures = if is_static {
            &mut class_info.static_methods
        } else {
            &mut class_info.methods
        };
        if let Some(signature) = signatures.get_mut(method_key) {
            ensure_variadic_for_func_args(signature);
        }
    }
}
