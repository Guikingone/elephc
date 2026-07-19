//! Purpose:
//! Dispatches AST statement nodes into EIR instructions and CFG terminators.
//!
//! Called from:
//! - `crate::ir_lower::function` for main, user functions, and methods.
//!
//! Key details:
//! - Every `StmtKind` variant has an explicit lowering branch.
//! - Structured control flow creates EIR blocks; complex PHP runtime behavior
//!   uses high-level opcodes with conservative effects.

use std::collections::HashSet;

mod loop_types;
mod return_type_guard;

use crate::ir::{
    BlockId, CmpPredicate, Immediate, IrHeapKind, IrType, LocalKind, LocalSlotId, Op, Ownership,
    SwitchCase, Terminator,
};
use crate::ir_lower::context::{FinallyFrame, LoopCleanup, LoopFrame, LoweredValue, LoweringContext};
use crate::ir_lower::effects_lookup;
use crate::ir_lower::expr::{
    coerce_to_int_at_span, lower_callable_array_for_assignment, lower_closure_for_assignment, lower_expr,
    static_callable_binding_for_expr, store_value_into_temp, string_op_uses_scratch_storage,
    type_satisfies_array_access_for_ir,
};
use crate::names::{php_symbol_key, property_hook_set_method};
use crate::parser::ast::{
    BinOp, CatchClause, Expr, ExprKind, InstanceOfTarget, StaticReceiver, Stmt, StmtKind,
};
use crate::span::Span;
use crate::types::PhpType;

/// Lowers one AST statement into the current EIR insertion block.
pub(crate) fn lower_stmt(ctx: &mut LoweringContext<'_, '_>, stmt: &Stmt) {
    if ctx.builder.insertion_block_is_terminated() {
        // A `label:` opens a new block that a `goto` elsewhere may branch into, so it must be lowered
        // even when the straight-line predecessor already terminated (e.g. the statement right before
        // it was a `goto`/`return`). Every other statement after a terminator is genuinely unreachable
        // and is skipped. Lowering the label repositions emission at its (reachable) block.
        if let StmtKind::Label(label) = &stmt.kind {
            lower_label(ctx, label);
        }
        return;
    }
    lower_statement_concat_reset(ctx, stmt.span);
    match &stmt.kind {
        StmtKind::Echo(expr) => lower_echo(ctx, expr, stmt.span),
        StmtKind::Assign { name, value } => lower_assign(ctx, name, value, stmt.span),
        StmtKind::RefAssign { target, source } => lower_ref_assign(ctx, target, source, stmt.span),
        StmtKind::RefAssignToTarget { target, source, append } => {
            lower_ref_assign_to_target(ctx, target, source, *append, stmt.span)
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => lower_if(ctx, condition, then_body, elseif_clauses, else_body.as_deref(), stmt.span),
        StmtKind::IfDef {
            symbol,
            then_body,
            else_body,
        } => lower_ifdef(ctx, symbol, then_body, else_body.as_deref(), stmt.span),
        StmtKind::While { condition, body } => lower_while(ctx, condition, body),
        StmtKind::DoWhile { body, condition } => lower_do_while(ctx, body, condition),
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => lower_for(ctx, init.as_deref(), condition.as_ref(), update.as_deref(), body),
        StmtKind::ArrayAssign { array, index, value } => {
            lower_array_assign(ctx, array, index, value, stmt.span);
        }
        StmtKind::NestedArrayAssign { target, value } => {
            lower_nested_array_assign(ctx, target, value, stmt.span);
        }
        StmtKind::ArrayPush { array, value } => lower_array_push(ctx, array, value, stmt.span),
        StmtKind::TypedAssign {
            type_expr,
            name,
            value,
        } => lower_typed_assign(ctx, type_expr, name, value, stmt.span),
        StmtKind::Foreach {
            array,
            key_var,
            value_var,
            value_by_ref,
            body,
        } => lower_foreach(ctx, array, key_var.as_deref(), value_var, *value_by_ref, body),
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => lower_switch(ctx, subject, cases, default.as_deref()),
        StmtKind::Include {
            path,
            once,
            required,
        } => lower_include(ctx, path, *once, *required, stmt.span),
        StmtKind::IncludeOnceMark { label } => lower_include_once_mark(ctx, label, stmt.span),
        StmtKind::IncludeOnceGuard { label, body } => {
            lower_include_once_guard(ctx, label, body, stmt.span);
        }
        StmtKind::Throw(expr) => lower_throw(ctx, expr),
        StmtKind::Synthetic(body) => lower_block(ctx, body),
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => lower_try(ctx, try_body, catches, finally_body.as_deref(), stmt.span),
        StmtKind::Break(level) => lower_break(ctx, *level),
        StmtKind::Continue(level) => lower_continue(ctx, *level),
        StmtKind::Goto(label) => lower_goto(ctx, label),
        StmtKind::Label(label) => lower_label(ctx, label),
        StmtKind::ExprStmt(expr) => {
            let value = lower_expr(ctx, expr);
            release_expr_statement_result(ctx, value, expr.span);
        }
        StmtKind::NamespaceDecl { name: _ } => lower_noop(ctx, stmt.span),
        StmtKind::NamespaceBlock { name: _, body } => lower_block(ctx, body),
        StmtKind::UseDecl { imports: _ } => lower_noop(ctx, stmt.span),
        StmtKind::FunctionDecl { .. }
        | StmtKind::ClassDecl { .. }
        | StmtKind::EnumDecl { .. }
        | StmtKind::PackedClassDecl { .. }
        | StmtKind::InterfaceDecl { .. }
        | StmtKind::TraitDecl { .. }
        | StmtKind::ExternFunctionDecl { .. }
        | StmtKind::ExternClassDecl { .. }
        | StmtKind::ExternGlobalDecl { .. } => lower_noop(ctx, stmt.span),
        StmtKind::FunctionVariantGroup { name, variants } => {
            lower_function_variant_group(ctx, name, variants, stmt.span);
        }
        StmtKind::FunctionVariantMark { name, variant } => {
            lower_function_variant_mark(ctx, name, variant, stmt.span);
        }
        StmtKind::Return(value) => lower_return(ctx, value.as_ref(), stmt.span),
        StmtKind::ConstDecl { name, value } => lower_const_decl(ctx, name, value, stmt.span),
        StmtKind::ListUnpack { vars, value } => lower_list_unpack(ctx, vars, value, stmt.span),
        StmtKind::Global { vars } => lower_global(ctx, vars),
        StmtKind::StaticVar { name, init } => lower_static_var(ctx, name, init, stmt.span),
        StmtKind::PropertyAssign {
            object,
            property,
            value,
        } => lower_property_assign(ctx, object, property, value, stmt.span),
        StmtKind::StaticPropertyAssign {
            receiver,
            property,
            value,
        } => lower_static_property_assign(ctx, receiver, property, value, stmt.span),
        StmtKind::StaticPropertyArrayPush {
            receiver,
            property,
            value,
        } => lower_static_property_array_push(ctx, receiver, property, value, stmt.span),
        StmtKind::StaticPropertyArrayAssign {
            receiver,
            property,
            index,
            value,
        } => lower_static_property_array_assign(ctx, receiver, property, index, value, stmt.span),
        StmtKind::DynamicStaticPropertyWrite {
            receiver,
            property,
            index,
            append,
            value,
        } => lower_dynamic_static_property_write(
            ctx,
            receiver,
            property,
            index.as_ref(),
            *append,
            value,
            stmt.span,
        ),
        StmtKind::PropertyArrayPush {
            object,
            property,
            value,
        } => lower_property_array_push(ctx, object, property, value, stmt.span),
        StmtKind::PropertyArrayAssign {
            object,
            property,
            index,
            value,
        } => lower_property_array_assign(ctx, object, property, index, value, stmt.span),
    }
}

/// Releases a discarded expression-statement result when it may own temporary storage.
fn release_expr_statement_result(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) {
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
}

/// Emits the statement-boundary concat-buffer reset expected by the ASM backend.
fn lower_statement_concat_reset(ctx: &mut LoweringContext<'_, '_>, span: Span) {
    if span.line == 0 {
        return;
    }
    ctx.emit_void(
        Op::ConcatReset,
        vec![],
        None,
        Op::ConcatReset.default_effects(),
        Some(span),
    );
}

/// Lowers a sequence of statements until the current block terminates.
///
/// Folds an adjacent `static $x; $x ??= <default>;` pair (see
/// `fold_static_null_coalesce_pair`) into a single once-guarded static init before dispatching
/// to `lower_stmt`, so this seam applies uniformly to every statement list a function-like body
/// can contain (top-level bodies route through here too — see
/// `crate::ir_lower::function::lower_body_into_function`).
pub(crate) fn lower_block(ctx: &mut LoweringContext<'_, '_>, body: &[Stmt]) {
    let mut index = 0;
    while index < body.len() {
        let stmt = &body[index];
        // Once the current block is terminated the remaining statements are unreachable in straight-
        // line order — except a `label:`, which a `goto` may branch into. Skip unreachable non-label
        // statements, but still lower labels so their block is opened and emission resumes there.
        if ctx.builder.insertion_block_is_terminated()
            && !matches!(stmt.kind, StmtKind::Label(_))
        {
            index += 1;
            continue;
        }
        if let Some((name, default, span)) = fold_static_null_coalesce_pair(body, index) {
            lower_static_var(ctx, name, default, span);
            index += 2;
            continue;
        }
        lower_stmt(ctx, stmt);
        index += 1;
    }
}

/// Detects the `static $x; $x ??= <default>;` idiom at `body[index]`/`body[index + 1]` and
/// returns `($x's name, <default>, the static declaration's span)` when it is safe to fold into a
/// single once-guarded static init (`static $x = <default>;`).
///
/// PHP requires `static $x = <expr>;` initializers to be compile-time constants, so PHP code uses
/// the bare-declaration-then-`??=` idiom (see `Symfony\Component\Cache\Traits\ContractsTrait::doGet`'s
/// `static $setMetadata; $setMetadata ??= \Closure::bind(...);`, the motivating example for this
/// fold) to assign an arbitrary expression exactly once and keep it across calls. `static $x;`
/// and `static $x = null;` are both PHP-equivalent, so this fold applies to either spelling — the
/// checker binds `$x`'s declaration-site type to `PhpType::Void` for both. Lowering the pair
/// separately later widens the static local's frame slot to `<default>`'s type via
/// `Op::StoreLocal`/`widen_local_storage_type` AFTER the `Op::InitStaticLocal` for the (now
/// stale, `Void`-typed) null init already emitted — producing a `Void`-into-`<slot type>`
/// mismatch the backend correctly rejects. Worse, `$x ??= <default>`'s `IsNull` check is
/// evaluated once at compile time against $x's flow-sensitive type, which is `Void` at this
/// program point on EVERY call (not just the first) — `is_null` on a `Void`-typed value is a
/// compile-time-constant `true` (see
/// `crate::codegen_ir::lower_inst::predicates::emit_is_null_result`), so the un-folded lowering
/// would silently re-evaluate `<default>` on every call, breaking "assign once" persistence for
/// any `<default>` with side effects or object identity. Folding into `Op::InitStaticLocal`
/// sidesteps that specific problem by reusing the once-flag-guarded machinery that typed statics
/// (`static $x = 5;`) already use correctly.
///
/// Only folds when this gate holds:
/// - `<default>` cannot itself evaluate to PHP null (`static_var_default_never_null`): real PHP
///   retries `??=`'s default on every call while the static stays null, which the fold's single
///   once-guarded write cannot reproduce, so an unprovably-nullable default is left for the
///   existing (loud) `init_static_local` backend error instead of silently diverging.
///
/// A second and third gate — "`<default>`'s (re-)evaluation cannot be observed" and "cannot leak
/// heap memory" — used to be required here because `Op::InitStaticLocal`'s codegen only guarded
/// the final *store into the persistent slot* behind the once-flag branch: the value-producing
/// instructions that compute `<default>` were separate, straight-line instructions emitted
/// *before* `Op::InitStaticLocal` in the same block, so they ran unconditionally on every call
/// regardless of the guard (verified empirically: a side-effecting constructor's `echo` printed 3
/// times across 3 calls, and closures with captures leaked heap blocks across calls even with no
/// observable side effect). `crate::ir_lower::stmt::lower_static_var` now wraps the WHOLE
/// initializer evaluation (not just the store) in an EIR-level once-guard `CondBr`, so those two
/// gates are no longer needed: a side-effecting or heap-allocating `<default>` now runs exactly
/// once across calls, matching PHP.
fn fold_static_null_coalesce_pair<'a>(
    body: &'a [Stmt],
    index: usize,
) -> Option<(&'a str, &'a Expr, Span)> {
    let first = body.get(index)?;
    let StmtKind::StaticVar { name, init } = &first.kind else {
        return None;
    };
    if !matches!(init.kind, ExprKind::Null) {
        return None;
    }
    let second = body.get(index + 1)?;
    let StmtKind::Assign {
        name: assign_name,
        value,
    } = &second.kind
    else {
        return None;
    };
    if assign_name != name {
        return None;
    }
    let ExprKind::NullCoalesce { value: current, default } = &value.kind else {
        return None;
    };
    let ExprKind::Variable(current_name) = &current.kind else {
        return None;
    };
    if current_name != name {
        return None;
    }
    if !static_var_default_never_null(default) {
        return None;
    }
    Some((name.as_str(), default.as_ref(), first.span))
}

/// Returns true when `expr`'s syntactic shape guarantees it can never evaluate to PHP null.
///
/// This is a conservative whitelist, not a full type inference: it only recognizes expression
/// kinds that are structurally incapable of producing null, so callers that gate soundness-
/// sensitive folds on it (see `fold_static_null_coalesce_pair`) never silently misclassify a
/// nullable expression as safe.
///
/// `ExprKind::Closure` (any closure literal — arrow or block-bodied, static or instance) always
/// evaluates to a `Closure` object, never null. `ExprKind::NewObject` (`new Foo(...)`) always
/// evaluates to an object or throws, never null. `\Closure::bind(...)` is handled separately by
/// `static_var_default_closure_bind_never_null`, since its "never null" proof depends on its
/// arguments' shape, not just its expression kind.
fn static_var_default_never_null(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::IntLiteral(_)
            | ExprKind::FloatLiteral(_)
            | ExprKind::StringLiteral(_)
            | ExprKind::BoolLiteral(_)
            | ExprKind::ArrayLiteral(_)
            | ExprKind::ArrayLiteralAssoc(_)
            | ExprKind::Closure { .. }
            | ExprKind::NewObject { .. }
    ) || static_var_default_closure_bind_never_null(expr)
}

/// Returns true when `expr` is a `\Closure::bind($closure, $newThis[, $scope])` call that is
/// PROVABLY non-null under elephc's closed-world compilation model.
///
/// PHP's `Closure::bind` returns `?Closure`: it returns null (with an `E_WARNING`) exactly when
/// (a) the scope class name cannot be resolved, or (b) a non-null `$newThis` is bound onto a
/// `static` closure (php-verified: `Closure::bind(static function(){}, null, C::class)` succeeds;
/// `Closure::bind(static function(){}, new C(), C::class)` warns and returns null; a non-static
/// closure never fails this way for any `$newThis`/scope combination, php-verified with a
/// scope-mismatched `$newThis` and a same-scope one, both non-null). Failure mode (a) cannot
/// occur here: `crate::types::checker`'s `validate_class_constant_receiver` already rejects an
/// unresolvable `X::class` receiver earlier in the pipeline, so any `X::class` scope
/// argument surviving to `ir_lower` names a real, declared class. That leaves failure mode (b) as
/// the only one this function needs to rule out syntactically: the closure literal must not be
/// `static`, OR (if it is) `$newThis` must be the literal `null` (PHP's own default when omitted).
fn static_var_default_closure_bind_never_null(expr: &Expr) -> bool {
    let ExprKind::StaticMethodCall { receiver, method, args } = &expr.kind else {
        return false;
    };
    let StaticReceiver::Named(class_name) = receiver else {
        return false;
    };
    if class_name.as_str().trim_start_matches('\\') != "Closure" || php_symbol_key(method) != "bind" {
        return false;
    }
    let Some(closure_arg) = args.first() else {
        return false;
    };
    let ExprKind::Closure { is_static, .. } = &closure_arg.kind else {
        return false;
    };
    if *is_static {
        let new_this_is_null = matches!(args.get(1).map(|arg| &arg.kind), None | Some(ExprKind::Null));
        if !new_this_is_null {
            return false;
        }
    }
    // Third arg (`$scope`), if present, must be a compiler-resolvable class-constant reference —
    // `validate_class_constant_receiver` already proved it names a real declared class, foreclosing
    // Closure::bind's "class not found" failure mode. `null`/omitted scope keeps the closure's
    // original scope, which is always valid.
    matches!(
        args.get(2).map(|arg| &arg.kind),
        None | Some(ExprKind::Null) | Some(ExprKind::ClassConstant { .. })
    )
}

/// Emits EIR for `echo`.
fn lower_echo(ctx: &mut LoweringContext<'_, '_>, expr: &Expr, span: Span) {
    let value = lower_expr(ctx, expr);
    ctx.emit_void(
        Op::EchoValue,
        vec![value.value],
        None,
        Op::EchoValue.default_effects(),
        Some(span),
    );
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
}

/// Lowers a plain PHP local assignment.
fn lower_assign(ctx: &mut LoweringContext<'_, '_>, name: &str, value: &Expr, span: Span) {
    // A by-reference `Closure::bind(fn &() => $this->prop, $obj, $obj)` assigned to a variable is
    // tracked as a static callable, like a closure literal, so a later `$b()` lowers to a direct
    // call that carries the property's reference-cell pointer instead of boxing it.
    let bound_closure = crate::ir_lower::expr::is_bound_closure_assignment_shape(ctx, value);
    let direct_closure = matches!(value.kind, ExprKind::Closure { .. }) || bound_closure;
    ctx.clear_pending_static_callable_result();
    let static_callable = static_callable_binding_for_expr(ctx, value);
    let fiber_start_sig = crate::ir_lower::fibers::start_sig_for_expr(ctx, value);
    let callable_array = lower_callable_array_for_assignment(ctx, value, static_callable.as_ref());
    let lowered = callable_array
        .as_ref()
        .map(|assignment| assignment.value)
        .or_else(|| lower_closure_for_assignment(ctx, name, value))
        .or_else(|| bound_closure.then(|| crate::ir_lower::expr::lower_bound_closure_for_assignment(ctx, value)).flatten())
        .unwrap_or_else(|| lower_expr(ctx, value));
    let (lowered, php_type) = contextualize_array_assignment(ctx, name, value, lowered, span);
    ctx.store_local(name, lowered, php_type, Some(span));
    let callable_result = if direct_closure {
        ctx.take_pending_static_callable_result()
    } else {
        ctx.clear_pending_static_callable_result();
        None
    };
    let static_callable = callable_array
        .map(|assignment| assignment.target)
        .or(static_callable)
        .or(callable_result);
    if let Some(target) = static_callable {
        ctx.bind_static_callable_local(name, target);
    }
    if let Some(sig) = fiber_start_sig {
        ctx.bind_fiber_start_sig(name, sig);
    }
}

/// Converts indexed array literals to hash storage when checker facts require an assoc local.
fn contextualize_array_assignment(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    value: &Expr,
    lowered: LoweredValue,
    span: Span,
) -> (LoweredValue, PhpType) {
    let php_type = ctx.builder.value_php_type(lowered.value);
    if !matches!(value.kind, ExprKind::ArrayLiteral(_)) {
        return (lowered, php_type);
    }
    if !matches!(php_type.codegen_repr(), PhpType::Array(_)) {
        return (lowered, php_type);
    }
    let contextual_ty = ctx.local_type(name).codegen_repr();
    if !matches!(contextual_ty, PhpType::AssocArray { .. }) {
        return (lowered, php_type);
    }
    let hash = ctx.emit_value(
        Op::ArrayToHash,
        vec![lowered.value],
        None,
        contextual_ty.clone(),
        Op::ArrayToHash.default_effects(),
        Some(span),
    );
    (hash, contextual_ty)
}

/// Lowers a by-reference assignment, dispatching on the kind of reference source.
///
/// - `$a = &$b` aliases two locals to one ref-cell.
/// - `$a = &$obj->prop` binds the local to the object's reference-property cell (write-through).
/// - `$a = &call()` binds the local to the cell returned by a by-reference callee.
fn lower_ref_assign(ctx: &mut LoweringContext<'_, '_>, target: &str, source: &Expr, span: Span) {
    match &source.kind {
        ExprKind::Variable(source_name) => {
            let fiber_start_sig = ctx.fiber_start_sig_for_local(source_name);
            ctx.alias_local_ref_cell(target, source_name, Some(span));
            if let Some(sig) = fiber_start_sig {
                ctx.bind_fiber_start_sig(target, sig);
            }
        }
        ExprKind::PropertyAccess { .. } => {
            crate::ir_lower::expr::lower_ref_assign_property(ctx, target, source, span);
        }
        ExprKind::DynamicPropertyAccess { .. } => {
            crate::ir_lower::expr::lower_ref_assign_dynamic_property(ctx, target, source, span);
        }
        ExprKind::StaticPropertyAccess { .. } => {
            crate::ir_lower::expr::lower_ref_assign_static_property(ctx, target, source, span);
        }
        ExprKind::FunctionCall { .. }
        | ExprKind::MethodCall { .. }
        | ExprKind::StaticMethodCall { .. }
        | ExprKind::ClosureCall { .. }
        | ExprKind::ExprCall { .. } => {
            crate::ir_lower::expr::lower_ref_assign_call(ctx, target, source, span);
        }
        ExprKind::ArrayAccess { array, index } => {
            lower_ref_assign_array_element(ctx, target, array, index, span);
        }
        _ => {
            // Other source shapes are rejected by the checker; evaluate for side effects.
            lower_expr(ctx, source);
        }
    }
}

/// Lowers `$target = &$arr[$k]`: promote the hash element to a kind-6 reference cell (via
/// `HashRefElement`) and bind `$target` as an owning alias to that cell (via `adopt_ref_cell`).
///
/// Indexed arrays cannot carry a per-element reference tag, so a plain-variable indexed array is
/// promoted to a hash first and stored back to its local, matching Zend's de-packing behavior.
fn lower_ref_assign_array_element(
    ctx: &mut LoweringContext<'_, '_>,
    target: &str,
    array: &Expr,
    index: &Expr,
    span: Span,
) {
    let array_value = lower_expr(ctx, array);
    // For an indexed array, promote to a hash first (Zend de-packs on reference-take). Defer the
    // write-back of the promoted hash to its local until after `HashRefElement`, matching the clean
    // ordering of the string-key promotion path (store the container last, once fully mutated).
    let (hash_value, promotion) = if let IrType::Heap(IrHeapKind::Array) = array_value.ir_type {
        if let ExprKind::Variable(array_name) = &array.kind {
            let current_ty = ctx.builder.value_php_type(array_value.value);
            let element_ty = reference_element_type(&current_ty);
            let assoc_ty = promoted_assoc_array_type(current_ty, element_ty);
            let hash = ctx.emit_value(
                Op::ArrayToHash,
                vec![array_value.value],
                None,
                assoc_ty.clone(),
                Op::ArrayToHash.default_effects(),
                Some(span),
            );
            (hash, Some((array_name.clone(), assoc_ty)))
        } else {
            (array_value, None)
        }
    } else {
        (array_value, None)
    };
    let index_value = lower_expr(ctx, index);
    let element_ty = reference_element_type(&ctx.builder.value_php_type(hash_value.value));
    let cell_ptr = ctx.emit_value(
        Op::HashRefElement,
        vec![hash_value.value, index_value.value],
        None,
        element_ty.clone(),
        Op::HashRefElement.default_effects(),
        Some(span),
    );
    // `HashRefElement` left the promoted (possibly relocated) hash in `hash_value`'s home, so store
    // it back to the array local now.
    if let Some((array_name, assoc_ty)) = promotion {
        ctx.store_mutated_local(&array_name, hash_value, assoc_ty.clone(), Some(span));
        // The promotion is an authoritative representation change (indexed Array → hash): force the
        // slot's storage type to the promoted AssocArray so scope-exit cleanup releases it with
        // `__rt_decref_hash`. `store_mutated_local` widens Array + AssocArray to `Mixed`, which
        // would free the raw hash with `__rt_decref_mixed` and leak it.
        ctx.set_local_type_exact(&array_name, assoc_ty);
    }
    ctx.adopt_ref_cell(target, cell_ptr, element_ty, Some(span));
}

/// Returns the referenced element's PHP type for a hash/array container (defaulting to `Mixed`).
///
/// An EMPTY container (`Never`/`Void` element type, e.g. a fresh `$t = []` de-packed by the
/// first `$t[$k] = &$v`) widens to `Mixed`: a reference-bound element is always read back
/// through the Mixed tag-dispatch path, and stamping the promoted hash's value type `Void`
/// would make every later element read an unsupported `hash_get` of `Void`.
fn reference_element_type(container: &PhpType) -> PhpType {
    let element_ty = match container.codegen_repr() {
        PhpType::AssocArray { value, .. } => (*value).clone(),
        PhpType::Array(element) => (*element).clone(),
        _ => PhpType::Mixed,
    };
    if is_empty_indexed_array_element(&element_ty) {
        PhpType::Mixed
    } else {
        element_ty
    }
}

/// Lowers a by-reference assignment whose left-hand side is a property or array element.
///
/// - `$obj->prop = &$src->q` (property source): the source property's ref-cell pointer is
///   stored into the target property's slot, so both properties share one cell (forward bind).
/// - `$obj->prop = &$src` (variable/call source): the value is first written into the
///   target property's owned cell, then the source local is aliased to that cell (reverse
///   bind), matching `$obj->prop = $src; $src = &$obj->prop;`. This keeps the cell owned by
///   the object (freed at destruction) while the local borrows it, avoiding a double free.
/// - Array-element targets are rejected by the checker; lowering only evaluates the source
///   for side effects to stay total.
fn lower_ref_assign_to_target(
    ctx: &mut LoweringContext<'_, '_>,
    target: &Expr,
    source: &Expr,
    append: bool,
    span: Span,
) {
    // Append targets (`$a[] = &$var`, `$a[$k][] = &$var`) name the CONTAINER. The checker has
    // already rejected static/instance-property append containers, so only a plain LOCAL array
    // variable (flat) or a nested LOCAL array element (whose base is a plain variable) reach here.
    if append {
        lower_ref_assign_local_array_element(ctx, target, source, true, span);
        return;
    }
    match &target.kind {
        ExprKind::PropertyAccess { object, property } => match &source.kind {
            ExprKind::PropertyAccess { .. } => {
                crate::ir_lower::expr::lower_bind_prop_ref_cell(ctx, object, property, source, span);
            }
            ExprKind::Variable(source_name) => {
                lower_property_assign(ctx, object, property, source, span);
                crate::ir_lower::expr::lower_ref_assign_property(ctx, source_name, target, span);
            }
            _ => {
                // A by-reference call source: write its value through the property cell.
                lower_property_assign(ctx, object, property, source, span);
            }
        },
        // `self::$a[$dir] = &self::$a[$k]`: aliasing a static-property array element. The checker has
        // validated both operands (same static array) and de-packed the property to a hash type.
        ExprKind::ArrayAccess { array, index }
            if matches!(array.kind, ExprKind::StaticPropertyAccess { .. }) =>
        {
            if let ExprKind::StaticPropertyAccess { receiver, property } = &array.kind {
                lower_ref_assign_static_prop_element(
                    ctx, receiver, property, index, source, span,
                );
            }
        }
        // `$a[$k] = &$var`: aliasing an explicit-key element of a plain LOCAL array to a
        // plain-variable source (checker-validated + de-packed).
        ExprKind::ArrayAccess { array, .. } if matches!(array.kind, ExprKind::Variable(_)) => {
            lower_ref_assign_local_array_element(ctx, target, source, false, span);
        }
        _ => {
            lower_expr(ctx, source);
        }
    }
}

/// Lowers `self::$a[$dir] = &self::$a[$k]`: reference-binds one element of a static-property array
/// as an alias of another element of the SAME static array (SLICE 2/3, the DebugClassLoader gate).
///
/// The target static property has been de-packed to a hash type by the checker, so it is loaded
/// once as the promoted `AssocArray`. The source element `&self::$a[$k]` is promoted to a kind-6
/// reference cell via `HashRefElement` (reusing SLICE 1's machinery over the static-prop hash), and
/// that cell is bound into `hash[$dir]` via the new `HashBindRefElement`. The SAME loaded hash is
/// threaded load → (ArrayToHash) → HashRefElement($k) → HashBindRefElement($dir) → store, so codegen
/// suppresses the store's `release_previous` for this round-trip of the same container.
fn lower_ref_assign_static_prop_element(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    target_index: &Expr,
    source: &Expr,
    span: Span,
) {
    // The checker guarantees the source is `&self::$SRC[$k]` naming the SAME static property.
    let ExprKind::ArrayAccess {
        index: source_index,
        ..
    } = &source.kind
    else {
        // Defensive: the checker rejects any other source shape; evaluate for side effects.
        lower_expr(ctx, source);
        return;
    };

    // Load the (de-packed) static-property array once as a hash. If the runtime value is still an
    // indexed array, promote it in place with `ArrayToHash` (Zend de-packs on reference-take).
    let hash_ty = static_property_type(ctx, receiver, property).unwrap_or(PhpType::Mixed);
    let mut hash = load_static_property_as(ctx, receiver, property, hash_ty, span);
    if let IrType::Heap(IrHeapKind::Array) = hash.ir_type {
        let current_ty = ctx.builder.value_php_type(hash.value);
        let element_ty = reference_element_type(&current_ty);
        let assoc_ty = promoted_assoc_array_type(current_ty, element_ty);
        hash = ctx.emit_value(
            Op::ArrayToHash,
            vec![hash.value],
            None,
            assoc_ty,
            Op::ArrayToHash.default_effects(),
            Some(span),
        );
    }

    // SOURCE `&self::$a[$k]`: promote the element at `$k` to a kind-6 reference cell over the SAME
    // loaded hash, threading the possibly-relocated hash forward.
    let element_ty = reference_element_type(&ctx.builder.value_php_type(hash.value));
    let source_key = lower_expr(ctx, source_index);
    let cell = ctx.emit_value(
        Op::HashRefElement,
        vec![hash.value, source_key.value],
        None,
        element_ty,
        Op::HashRefElement.default_effects(),
        Some(span),
    );
    let hash_ty_after = ctx.builder.value_php_type(hash.value);

    // TARGET `self::$a[$dir]`: bind the shared cell into `hash[$dir]` with value-tag 11, threading
    // the final relocated hash back for the store.
    let target_key = lower_expr(ctx, target_index);
    let bound_hash = ctx.emit_value(
        Op::HashBindRefElement,
        vec![hash.value, target_key.value, cell.value],
        None,
        hash_ty_after,
        Op::HashBindRefElement.default_effects(),
        Some(span),
    );

    // Store the mutated container back. Codegen suppresses `release_previous` because the stored
    // value traces back to the just-loaded static property (same container round-trip).
    store_static_property(ctx, receiver, property, bound_hash.value, span);
}

/// Lowers a reference alias INTO a LOCAL array element whose source is a plain variable:
/// `$a[$k] = &$var` (explicit key, `append == false`), `$a[] = &$var` (flat append,
/// `append == true`), or `$loops[$k][] = &$var` (nested append, `append == true` with an
/// `ArrayAccess` container).
///
/// Every form uses the same REVERSE-BIND shape the checker validated: the source variable's value is
/// materialized into the element's kind-6 reference cell, then the source local adopts that cell
/// (`adopt_ref_cell`), which releases the local's prior direct share and increfs the cell so the
/// element (alias) and the local (owner) each hold one share. The container is de-packed to a hash so
/// it can carry a per-element reference tag, and the possibly-relocated hash is written back to the
/// local. The source is checker-guaranteed to be a plain variable.
fn lower_ref_assign_local_array_element(
    ctx: &mut LoweringContext<'_, '_>,
    target: &Expr,
    source: &Expr,
    append: bool,
    span: Span,
) {
    let ExprKind::Variable(var_name) = &source.kind else {
        // Defensive: the checker rejects any non-variable source; evaluate for side effects.
        lower_expr(ctx, source);
        return;
    };
    match (&target.kind, append) {
        // `$a[$k] = &$var`: explicit-key alias into a flat local array.
        (ExprKind::ArrayAccess { array, index }, false)
            if matches!(array.kind, ExprKind::Variable(_)) =>
        {
            if let ExprKind::Variable(array_name) = &array.kind {
                lower_ref_assign_local_element_explicit_key(ctx, array_name, index, var_name, span);
            }
        }
        // `$a[] = &$var`: append a reference into a flat local array.
        (ExprKind::Variable(array_name), true) => {
            lower_ref_append_into_local_hash(ctx, array_name, var_name, span);
        }
        // `$loops[$k][] = &$var`: append a reference into a nested local array element.
        (ExprKind::ArrayAccess { array, index }, true)
            if matches!(array.kind, ExprKind::Variable(_)) =>
        {
            if let ExprKind::Variable(outer_name) = &array.kind {
                lower_ref_assign_local_nested_append(ctx, outer_name, index, var_name, span);
            }
        }
        _ => {
            // Defensive: the checker rejects any other target shape; evaluate for side effects.
            lower_expr(ctx, source);
        }
    }
}

/// Loads a local array as a hash, de-packing an indexed array in place (Zend de-packs on
/// reference-take). Returns the loaded hash value plus an optional `(name, promoted_type)` write-back
/// obligation to run after the container has been fully mutated (mirroring
/// `lower_ref_assign_array_element`).
fn load_local_array_as_hash(
    ctx: &mut LoweringContext<'_, '_>,
    array_name: &str,
    span: Span,
) -> (LoweredValue, Option<(String, PhpType)>) {
    let array_value = ctx.load_local(array_name, Some(span));
    if let IrType::Heap(IrHeapKind::Array) = array_value.ir_type {
        let current_ty = ctx.builder.value_php_type(array_value.value);
        let element_ty = reference_element_type(&current_ty);
        let assoc_ty = promoted_assoc_array_type(current_ty, element_ty);
        let hash = ctx.emit_value(
            Op::ArrayToHash,
            vec![array_value.value],
            None,
            assoc_ty.clone(),
            Op::ArrayToHash.default_effects(),
            Some(span),
        );
        (hash, Some((array_name.to_string(), assoc_ty)))
    } else {
        (array_value, None)
    }
}

/// Appends a reference to `$var` into the hash held by the LOCAL `array_name` (`$a[] = &$var`, and the
/// per-level primitive reused by the nested form on a temporary inner hash).
///
/// Get-or-promotes `$var`'s PERSISTENT kind-6 cell (shared across every bind — Zend semantics), then
/// appends THAT cell into the hash at the next int key with an incref (`HashRefAppendElement`), and
/// writes the possibly-relocated hash back to the local.
fn lower_ref_append_into_local_hash(
    ctx: &mut LoweringContext<'_, '_>,
    array_name: &str,
    var_name: &str,
    span: Span,
) {
    // Bind `$var`'s ONE persistent cell (marks `$var` a reference; idempotent for loop bodies).
    // The alias keeps the original element type (checker `check_ref_assign_local_array_element`):
    // the slot's storage type stays the pre-bind type, so `Op::LocalRefEnsure` carries the matching
    // runtime value-tag and the cell's inner-tag stays in sync with its inner value. A
    // type-changing reassign is read back through the ELEMENT value type (Mixed), not the alias —
    // `__rt_ref_cell_store` stamps the NEW value's runtime tag at `[cell+8]` independently.
    let cell = ctx.ensure_local_ref_cell(var_name, Some(span));
    let (hash, promotion) = load_local_array_as_hash(ctx, array_name, span);
    let hash_ty = ctx.builder.value_php_type(hash.value);
    let new_hash = ctx.emit_value(
        Op::HashRefAppendElement,
        vec![hash.value, cell.value],
        None,
        hash_ty,
        Op::HashRefAppendElement.default_effects(),
        Some(span),
    );
    // On the ArrayToHash (indexed-container) path the op cannot auto-store the relocated hash (it
    // traces to `ArrayToHash`, not `LoadLocal`), so write it back explicitly.
    if let Some((name, assoc_ty)) = promotion {
        ctx.store_mutated_local(&name, new_hash, assoc_ty.clone(), Some(span));
        ctx.set_local_type_exact(&name, assoc_ty);
    }
}

/// Lowers `$a[$k] = &$var`: aliases an explicit-key element of a flat local array to `$var`.
///
/// Get-or-promotes `$var`'s PERSISTENT kind-6 cell, then binds THAT cell into `hash[$k]` with an
/// incref (`HashBindRefElement`, value-tag 11), and writes the relocated hash back.
fn lower_ref_assign_local_element_explicit_key(
    ctx: &mut LoweringContext<'_, '_>,
    array_name: &str,
    index: &Expr,
    var_name: &str,
    span: Span,
) {
    // Bind `$var`'s ONE persistent cell (marks `$var` a reference; idempotent for loop bodies).
    // The alias keeps the original element type — see `lower_ref_append_into_local_hash` for the
    // rationale (the slot's storage type stays the pre-bind type so `Op::LocalRefEnsure`'s tag
    // matches the slot value; a type-changing reassign is read back through the Mixed element).
    let cell = ctx.ensure_local_ref_cell(var_name, Some(span));
    let (hash, promotion) = load_local_array_as_hash(ctx, array_name, span);
    let key = lower_expr(ctx, index);
    let hash_ty = ctx.builder.value_php_type(hash.value);
    let new_hash = ctx.emit_value(
        Op::HashBindRefElement,
        vec![hash.value, key.value, cell.value],
        None,
        hash_ty,
        Op::HashBindRefElement.default_effects(),
        Some(span),
    );
    if let Some((name, assoc_ty)) = promotion {
        ctx.store_mutated_local(&name, new_hash, assoc_ty.clone(), Some(span));
        ctx.set_local_type_exact(&name, assoc_ty);
    }
}

/// Lowers `$loops[$k][] = &$var`: appends a reference to `$var` into a NESTED local array element
/// (the `PhpDumper.php:459` gate).
///
/// The outer local is loaded once as a hash. The inner element `$loops[$k]` is produced into a hidden
/// temporary that OWNS a share (so the reference append copy-on-write splits it away from the outer,
/// side-stepping `refprop-nested-append-writethrough`): when the key already exists the inner hash is
/// read and retained, otherwise a fresh empty hash is vivified. The reference is appended into that
/// temporary (reusing the flat per-level primitive), then the relocated inner is explicitly written
/// back into `outer[$k]` and the relocated outer is stored to the local.
fn lower_ref_assign_local_nested_append(
    ctx: &mut LoweringContext<'_, '_>,
    outer_name: &str,
    index: &Expr,
    var_name: &str,
    span: Span,
) {
    let (outer, promotion) = load_local_array_as_hash(ctx, outer_name, span);
    let inner_hash_ty = reference_element_type(&ctx.builder.value_php_type(outer.value));
    let key = lower_expr(ctx, index);

    // Produce the inner hash into an owned hidden temp, vivifying an empty hash when the key is
    // absent. Owning a share forces the subsequent reference append to copy-on-write split the inner
    // away from `outer[$k]`, so the final write-back is not a same-pointer round-trip.
    let inner_temp = ctx.declare_owned_hidden_temp(inner_hash_ty.clone());
    let exists = ctx.emit_value(
        Op::HashIsset,
        vec![outer.value, key.value],
        None,
        PhpType::Bool,
        Op::HashIsset.default_effects(),
        Some(span),
    );
    let split_initialized = ctx.initialized_slots_snapshot();
    let present_block = ctx.builder.create_named_block("nref.present", Vec::new());
    let vivify_block = ctx.builder.create_named_block("nref.vivify", Vec::new());
    let merge = ctx.builder.create_named_block("nref.merge", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: exists.value,
        then_target: present_block,
        then_args: Vec::new(),
        else_target: vivify_block,
        else_args: Vec::new(),
    });

    // Present: read the existing inner element and retain it into the temp.
    ctx.builder.position_at_end(present_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    let existing = ctx.emit_value(
        Op::HashGet,
        vec![outer.value, key.value],
        None,
        inner_hash_ty.clone(),
        Op::HashGet.default_effects(),
        Some(span),
    );
    store_value_into_temp(ctx, &inner_temp, inner_hash_ty.clone(), existing, span);
    branch_to(ctx, merge);

    // Absent: vivify a fresh empty hash into the temp.
    ctx.builder.position_at_end(vivify_block);
    ctx.restore_initialized_slots(split_initialized);
    let vivified = ctx.emit_value(
        Op::HashNew,
        Vec::new(),
        Some(Immediate::Capacity(0)),
        inner_hash_ty.clone(),
        Op::HashNew.default_effects(),
        Some(span),
    );
    store_value_into_temp(ctx, &inner_temp, inner_hash_ty.clone(), vivified, span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);

    // Append the reference into the (now-uniquely-owned) inner temp and reverse-bind `$var`.
    lower_ref_append_into_local_hash(ctx, &inner_temp, var_name, span);

    // Write the relocated inner hash back into `outer[$k]`. `HashSet`'s value materialization retains
    // the borrowed inner (refcount 2: temp + `outer[$k]`); the temp's redundant share is then released
    // with the inner's ACTUAL hash type (not via `unset`, which would first widen the slot toward
    // `Void`/`Mixed` and decref through the wrong runtime helper — leaking the hash), leaving
    // `outer[$k]` the sole owner (refcount 1) so the container frees cleanly. The void `HashSet`
    // codegen writes the possibly-relocated outer hash back to its SSA home and — when `outer` traces
    // directly to a `LoadLocal` (already a hash, `promotion == None`) — to the outer local as well.
    let inner_final = ctx.load_local(&inner_temp, Some(span));
    ctx.emit_void(
        Op::HashSet,
        vec![outer.value, key.value, inner_final.value],
        None,
        Op::HashSet.default_effects(),
        Some(span),
    );
    let inner_slot = ctx.declare_local(&inner_temp, inner_hash_ty.clone());
    ctx.release_stored_local_value(&inner_temp, inner_slot, Some(span));
    ctx.clear_owned_hidden_temp(&inner_temp, Some(span));
    // Only the ArrayToHash (indexed-container) case needs an explicit write-back: there `outer`
    // traces to `ArrayToHash`, not a `LoadLocal`, so the `HashSet` codegen cannot auto-store it.
    if let Some((_, assoc_ty)) = promotion {
        ctx.store_mutated_local(outer_name, outer, assoc_ty.clone(), Some(span));
        ctx.set_local_type_exact(outer_name, assoc_ty);
    }
}

/// Lowers an `if` / `elseif` / `else` chain and terminates unreachable merge blocks explicitly.
fn lower_if(
    ctx: &mut LoweringContext<'_, '_>,
    condition: &Expr,
    then_body: &[Stmt],
    elseif_clauses: &[(Expr, Vec<Stmt>)],
    else_body: Option<&[Stmt]>,
    span: Span,
) {
    let merge = ctx.builder.create_named_block("if.merge", Vec::new());
    let merge_reachable =
        lower_if_chain(ctx, condition, then_body, elseif_clauses, else_body, merge, span);
    ctx.builder.position_at_end(merge);
    if !merge_reachable {
        ctx.builder.terminate(Terminator::Unreachable);
    }
    ctx.clear_static_callable_locals();
}

/// Recursively emits one condition node in an `if` chain and reports whether the merge is reachable.
fn lower_if_chain(
    ctx: &mut LoweringContext<'_, '_>,
    condition: &Expr,
    then_body: &[Stmt],
    elseif_clauses: &[(Expr, Vec<Stmt>)],
    else_body: Option<&[Stmt]>,
    merge: BlockId,
    span: Span,
) -> bool {
    let cond_value = lower_expr(ctx, condition);
    let cond_value = ctx.truthy(cond_value, Some(condition.span));
    let split_initialized = ctx.initialized_slots_snapshot();
    let split_types = ctx.local_types_snapshot();
    let then_block = ctx.builder.create_named_block("if.then", Vec::new());
    let else_block = ctx.builder.create_named_block("if.else", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: cond_value.value,
        then_target: then_block,
        then_args: Vec::new(),
        else_target: else_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(then_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    ctx.restore_local_types(split_types.clone());
    apply_instanceof_then_narrowing(ctx, condition);
    lower_block(ctx, then_body);
    let then_initialized = ctx.initialized_slots_snapshot();
    let then_types = ctx.local_types_snapshot();
    let mut merge_reachable = false;
    let then_reachable = !ctx.builder.insertion_block_is_terminated();
    if then_reachable {
        merge_reachable = true;
        branch_to(ctx, merge);
    }

    ctx.clear_static_callable_locals();
    ctx.builder.position_at_end(else_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    ctx.restore_local_types(split_types.clone());
    let else_reachable = if let Some(((next_condition, next_body), rest)) = elseif_clauses.split_first() {
        lower_if_chain(ctx, next_condition, next_body, rest, else_body, merge, span)
    } else if let Some(else_body) = else_body {
        lower_block(ctx, else_body);
        if !ctx.builder.insertion_block_is_terminated() {
            branch_to(ctx, merge);
            true
        } else {
            false
        }
    } else {
        lower_noop(ctx, span);
        if !ctx.builder.insertion_block_is_terminated() {
            branch_to(ctx, merge);
            true
        } else {
            false
        }
    };
    merge_reachable |= else_reachable;
    let else_initialized = ctx.initialized_slots_snapshot();
    let else_types = ctx.local_types_snapshot();
    ctx.restore_initialized_slots(merge_initialized_slots(
        &split_initialized,
        then_initialized,
        then_reachable,
        else_initialized,
        else_reachable,
    ));
    let merged_types = merge_local_types(
        ctx,
        &split_types,
        then_types,
        then_reachable,
        else_types,
        else_reachable,
    );
    ctx.restore_local_types(merged_types);
    merge_reachable
}

/// Refines the flow-sensitive local types for an `if`'s then-branch using `instanceof` guards in
/// the condition, mirroring the type checker's narrowing so the EIR backend sees the concrete
/// class. Without this, a call to a method that exists only on the concrete subtype (accepted by
/// the checker via `instanceof` narrowing) is lowered on the declared interface type, resolves to
/// no signature, and falls back to `Mixed` — which mis-types by-reference-return ref-cell chains
/// and crashes. Handles `$var instanceof Class` directly and both operands of a top-level `&&`
/// chain; other guard shapes (member-path receivers, `||`, ternary) keep the runtime class-id
/// fallback and are out of scope here.
fn apply_instanceof_then_narrowing(ctx: &mut LoweringContext<'_, '_>, condition: &Expr) {
    match &condition.kind {
        ExprKind::InstanceOf { value, target: InstanceOfTarget::Name(name) } => {
            if let ExprKind::Variable(var) = &value.kind {
                let class_name = crate::ir_lower::expr::instanceof_target_name(ctx, name.as_str());
                // Only narrow to a statically-known CLASS (the off-interface-method case). Narrowing
                // to an interface would not resolve the method and is a needless de-refinement.
                if ctx.classes.contains_key(&class_name) {
                    ctx.set_local_type(var, PhpType::Object(class_name));
                }
            }
        }
        ExprKind::BinaryOp { left, op: BinOp::And, right } => {
            apply_instanceof_then_narrowing(ctx, left);
            apply_instanceof_then_narrowing(ctx, right);
        }
        _ => {}
    }
}

/// Merges the flow-sensitive local-type facts from the reachable branches of an `if`.
///
/// Only branches that reach the merge point contribute; a branch that returns, throws,
/// or otherwise cannot fall through must not leak its type mutations into the code after
/// the `if`. When a local's logical type differs between two reachable branches, the merged
/// type is the local's already-widened frame-storage type, which can hold either branch's
/// value. This keeps a `string` parameter reassigned to `int` on one branch from being read
/// with the wrong representation on a path where that reassignment never ran.
fn merge_local_types(
    ctx: &LoweringContext<'_, '_>,
    split_types: &crate::types::TypeEnv,
    then_types: crate::types::TypeEnv,
    then_reachable: bool,
    else_types: crate::types::TypeEnv,
    else_reachable: bool,
) -> crate::types::TypeEnv {
    match (then_reachable, else_reachable) {
        (true, false) => then_types,
        (false, true) => else_types,
        (false, false) => split_types.clone(),
        (true, true) => join_local_types(ctx, then_types, &else_types),
    }
}

/// Joins two reachable-path local-type environments into one that is valid after both paths merge.
///
/// A local present in both with the same logical type keeps that type. When the two paths disagree
/// on a local's type, the join adopts the local's already-widened frame-storage type (`Mixed` for
/// an incompatible reassignment), which can represent a value from either path. Locals present on
/// only one path are carried over unchanged. Used for `if`/`switch` merges so per-branch type
/// changes are combined instead of one path's facts silently overwriting the other's.
fn join_local_types(
    ctx: &LoweringContext<'_, '_>,
    mut base: crate::types::TypeEnv,
    other: &crate::types::TypeEnv,
) -> crate::types::TypeEnv {
    for (name, other_ty) in other {
        match base.get(name) {
            Some(base_ty) if base_ty == other_ty => {}
            Some(_) => {
                let widened = ctx
                    .local_storage_php_type(name)
                    .unwrap_or_else(|| other_ty.clone());
                base.insert(name.clone(), widened);
            }
            None => {
                base.insert(name.clone(), other_ty.clone());
            }
        }
    }
    base
}

/// Merges definitely-initialized locals from the reachable branches of an `if`.
fn merge_initialized_slots(
    split_initialized: &HashSet<LocalSlotId>,
    then_initialized: HashSet<LocalSlotId>,
    then_reachable: bool,
    else_initialized: HashSet<LocalSlotId>,
    else_reachable: bool,
) -> HashSet<LocalSlotId> {
    match (then_reachable, else_reachable) {
        (true, true) => then_initialized
            .intersection(&else_initialized)
            .copied()
            .collect(),
        (true, false) => then_initialized,
        (false, true) => else_initialized,
        (false, false) => split_initialized.clone(),
    }
}

/// Lowers a residual `ifdef`; normally the conditional pass removes these first.
fn lower_ifdef(
    ctx: &mut LoweringContext<'_, '_>,
    _symbol: &str,
    then_body: &[Stmt],
    else_body: Option<&[Stmt]>,
    _span: Span,
) {
    if !then_body.is_empty() {
        lower_block(ctx, then_body);
    } else if let Some(else_body) = else_body {
        lower_block(ctx, else_body);
    }
    ctx.clear_static_callable_locals();
}

/// Lowers a `while` loop.
fn lower_while(ctx: &mut LoweringContext<'_, '_>, condition: &Expr, body: &[Stmt]) {
    // A local reassigned inside the loop can be read on a later iteration by the condition or a
    // body statement placed before the reassignment; widen its type to Mixed for the whole loop
    // scope so those reads do not coerce a widened Mixed slot to a stale narrow type.
    loop_types::prewiden_loop_carried_locals(ctx, &[body], &[], &[condition]);
    let header = ctx.builder.create_named_block("while.cond", Vec::new());
    let body_block = ctx.builder.create_named_block("while.body", Vec::new());
    let exit = ctx.builder.create_named_block("while.exit", Vec::new());
    branch_to(ctx, header);

    ctx.builder.position_at_end(header);
    let cond = lower_expr(ctx, condition);
    let cond = ctx.truthy(cond, Some(condition.span));
    ctx.builder.terminate(Terminator::CondBr {
        cond: cond.value,
        then_target: body_block,
        then_args: Vec::new(),
        else_target: exit,
        else_args: Vec::new(),
    });

    ctx.clear_static_callable_locals();
    ctx.builder.position_at_end(body_block);
    ctx.loop_stack.push(LoopFrame {
        break_block: exit,
        continue_block: header,
        cleanup: None,
    });
    lower_block(ctx, body);
    ctx.loop_stack.pop();
    branch_to(ctx, header);
    ctx.builder.position_at_end(exit);
    ctx.clear_static_callable_locals();
}

/// Lowers a `do while` loop.
fn lower_do_while(ctx: &mut LoweringContext<'_, '_>, body: &[Stmt], condition: &Expr) {
    // See `lower_while`: pre-widen loop-carried locals so a read placed before an in-loop
    // reassignment is typed to match the widened Mixed slot on iterations past the first.
    loop_types::prewiden_loop_carried_locals(ctx, &[body], &[], &[condition]);
    let body_block = ctx.builder.create_named_block("do.body", Vec::new());
    let cond_block = ctx.builder.create_named_block("do.cond", Vec::new());
    let exit = ctx.builder.create_named_block("do.exit", Vec::new());
    branch_to(ctx, body_block);

    ctx.builder.position_at_end(body_block);
    ctx.loop_stack.push(LoopFrame {
        break_block: exit,
        continue_block: cond_block,
        cleanup: None,
    });
    lower_block(ctx, body);
    ctx.loop_stack.pop();
    branch_to(ctx, cond_block);

    ctx.builder.position_at_end(cond_block);
    let cond = lower_expr(ctx, condition);
    let cond = ctx.truthy(cond, Some(condition.span));
    ctx.builder.terminate(Terminator::CondBr {
        cond: cond.value,
        then_target: body_block,
        then_args: Vec::new(),
        else_target: exit,
        else_args: Vec::new(),
    });
    ctx.clear_static_callable_locals();
    ctx.builder.position_at_end(exit);
    ctx.clear_static_callable_locals();
}

/// Lowers a `for` loop.
fn lower_for(
    ctx: &mut LoweringContext<'_, '_>,
    init: Option<&Stmt>,
    condition: Option<&Expr>,
    update: Option<&Stmt>,
    body: &[Stmt],
) {
    if let Some(init) = init {
        lower_stmt(ctx, init);
    }
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
    // Pre-widen loop-carried locals after `init` (which fixes their entry types) and before the
    // condition/body/update are lowered, so later-iteration reads use the widened Mixed slot. The
    // one-shot `init` is not part of the carried scope; the recurring update and condition are.
    let mut prescan_stmts: Vec<&Stmt> = Vec::new();
    if let Some(update) = update {
        prescan_stmts.push(update);
    }
    let prescan_exprs: Vec<&Expr> = condition.into_iter().collect();
    loop_types::prewiden_loop_carried_locals(ctx, &[body], &prescan_stmts, &prescan_exprs);

    let header = ctx.builder.create_named_block("for.cond", Vec::new());
    let body_block = ctx.builder.create_named_block("for.body", Vec::new());
    let update_block = ctx.builder.create_named_block("for.update", Vec::new());
    let exit = ctx.builder.create_named_block("for.exit", Vec::new());
    branch_to(ctx, header);

    ctx.builder.position_at_end(header);
    let cond = if let Some(condition) = condition {
        let cond = lower_expr(ctx, condition);
        ctx.truthy(cond, Some(condition.span))
    } else {
        emit_const_bool(ctx, true, None)
    };
    ctx.builder.terminate(Terminator::CondBr {
        cond: cond.value,
        then_target: body_block,
        then_args: Vec::new(),
        else_target: exit,
        else_args: Vec::new(),
    });

    ctx.clear_static_callable_locals();
    ctx.builder.position_at_end(body_block);
    ctx.loop_stack.push(LoopFrame {
        break_block: exit,
        continue_block: update_block,
        cleanup: None,
    });
    lower_block(ctx, body);
    ctx.loop_stack.pop();
    branch_to(ctx, update_block);

    ctx.builder.position_at_end(update_block);
    if let Some(update) = update {
        lower_stmt(ctx, update);
    }
    branch_to(ctx, header);
    ctx.builder.position_at_end(exit);
    ctx.clear_static_callable_locals();
}

/// Releases the value operand of an array/hash element write when it is an owned
/// string. These writes PERSIST (copy) a string value into the container instead
/// of moving it (`__rt_str_persist`), so an owned string operand — e.g. a function
/// or extern call result like `$_ENV[$k] = getenv_value()` — would otherwise never
/// be freed (a per-write heap leak that exhausts the heap under `--web`). Non-string
/// refcounted values (objects, arrays) are moved, or retained only when borrowed,
/// by the write itself, so they must not be released here.
fn release_persisted_string_operand(ctx: &mut LoweringContext<'_, '_>, value: LoweredValue, span: Span) {
    let ty = ctx.builder.value_php_type(value.value);
    // Only release a FRESH owning string temporary (a call/concat result, etc.).
    // A borrowed load of a variable that still owns the string (e.g. the prelude's
    // `$_GET[$k] = $v`) must NOT be released here, or the container's stored copy
    // would be freed out from under it.
    if matches!(ty.codegen_repr(), PhpType::Str) && ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
}

/// Releases an indexed-array write operand when the backend retained or copied it.
pub(super) fn release_indexed_array_write_operand(
    ctx: &mut LoweringContext<'_, '_>,
    container_elem_ty: Option<&PhpType>,
    value: LoweredValue,
    span: Span,
) {
    if !ctx.value_is_owning_temporary(value) {
        return;
    }
    let value_ty = ctx.builder.value_php_type(value.value).codegen_repr();
    if matches!(
        container_elem_ty.map(PhpType::codegen_repr),
        Some(PhpType::Mixed)
    ) && !matches!(value_ty, PhpType::Mixed | PhpType::Union(_))
    {
        return;
    }
    crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
}

/// Returns the indexed-array element type in effect for a write.
pub(super) fn indexed_array_write_element_type(
    ctx: &LoweringContext<'_, '_>,
    array_value: LoweredValue,
    updated_ty: Option<&PhpType>,
) -> Option<PhpType> {
    let array_ty = updated_ty
        .cloned()
        .unwrap_or_else(|| ctx.builder.value_php_type(array_value.value));
    match array_ty.codegen_repr() {
        PhpType::Array(elem_ty) => Some(elem_ty.codegen_repr()),
        _ => None,
    }
}

/// Lowers an indexed array assignment.
fn lower_array_assign(
    ctx: &mut LoweringContext<'_, '_>,
    array: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
) {
    let array_value = ctx.load_local(array, Some(span));
    let mut index_value = lower_expr(ctx, index);
    let mut value_value = lower_expr(ctx, value);
    let op = array_set_op(array_value.ir_type);
    // A literal string index always means a hash key, so promote the destination
    // to associative storage like PHP. A boxed Mixed/Union index may hold either
    // an integer or a string key (foreach loop keys are always Mixed in EIR via
    // `Op::IterCurrentKey`), so it goes through `Op::ArraySetMixedKey`, whose
    // runtime helper keeps integer keys on indexed storage (preserving indexed
    // consumers like `implode`) and promotes only string keys to a hash. This
    // stops a `foreach($arr as $k=>$v) $dst[$k]=$v` rebuild from collapsing a
    // string key onto int 0. A foreach key over a concretely-indexed array is
    // known to be int-valued, so it is left on the coerce path to avoid
    // needlessly dispatching.
    if op == Op::ArraySet && index_value.ir_type == IrType::Str {
        lower_string_key_array_promotion(ctx, array, array_value, index_value, value_value, span);
        return;
    }
    if op == Op::ArraySet
        && index_is_boxed_mixed_key(index_value.ir_type)
        && !index_is_foreach_int_key(ctx, index)
    {
        lower_mixed_key_array_set(ctx, array, array_value, index_value, value_value, span);
        return;
    }
    if op == Op::ArraySet {
        index_value = coerce_to_int_at_span(ctx, index_value, Some(index.span));
        let array_ty = ctx.builder.value_php_type(array_value.value);
        value_value = coerce_indexed_array_set_value(ctx, &array_ty, value_value, Some(value.span));
    }
    if op == Op::ArraySet {
        let (array_value, updated_ty, needs_storeback) =
            prepare_indexed_array_local_set(ctx, array_value, value_value, span);
        ctx.emit_void(
            op,
            vec![array_value.value, index_value.value, value_value.value],
            None,
            op.default_effects(),
            Some(span),
        );
        let elem_ty = indexed_array_write_element_type(ctx, array_value, updated_ty.as_ref());
        finish_indexed_array_local_write(ctx, array, array_value, updated_ty, needs_storeback, span);
        release_indexed_array_write_operand(ctx, elem_ty.as_ref(), value_value, span);
        return;
    }
    ctx.emit_void(op, vec![array_value.value, index_value.value, value_value.value], None, op.default_effects(), Some(span));
    release_persisted_string_operand(ctx, index_value, span);
    release_persisted_string_operand(ctx, value_value, span);
}

/// Promotes an indexed local array to a Mixed-valued associative array for string-key writes.
fn lower_string_key_array_promotion(
    ctx: &mut LoweringContext<'_, '_>,
    array: &str,
    array_value: LoweredValue,
    index: LoweredValue,
    value: LoweredValue,
    span: Span,
) {
    let current_ty = ctx.builder.value_php_type(array_value.value);
    let value_ty = ctx.builder.value_php_type(value.value);
    let assoc_ty = promoted_assoc_array_type(current_ty, value_ty);
    let hash = ctx.emit_value(
        Op::ArrayToHash,
        vec![array_value.value],
        None,
        assoc_ty.clone(),
        Op::ArrayToHash.default_effects(),
        Some(span),
    );
    ctx.emit_void(
        Op::HashSet,
        vec![hash.value, index.value, value.value],
        None,
        Op::HashSet.default_effects(),
        Some(span),
    );
    release_persisted_string_operand(ctx, index, span);
    release_persisted_string_operand(ctx, value, span);
    ctx.store_mutated_local(array, hash, assoc_ty, Some(span));
}

/// Writes `value` into the indexed local `array` under a boxed Mixed/Union key.
///
/// The destination stays statically `Array(Mixed)` (so indexed consumers such as
/// `implode` keep routing to the indexed path) while `Op::ArraySetMixedKey`
/// dispatches the key tag at runtime: integer keys stay on indexed storage and
/// string keys promote the destination to a hash. This is the Mixed-key analogue
/// of `lower_string_key_array_promotion`, which unconditionally promotes because
/// a literal string key is always a hash key.
fn lower_mixed_key_array_set(
    ctx: &mut LoweringContext<'_, '_>,
    array: &str,
    array_value: LoweredValue,
    index: LoweredValue,
    value: LoweredValue,
    span: Span,
) {
    let mixed_array_ty = PhpType::Array(Box::new(PhpType::Mixed));
    let result = ctx.emit_value(
        Op::ArraySetMixedKey,
        vec![array_value.value, index.value, value.value],
        None,
        mixed_array_ty.clone(),
        Op::ArraySetMixedKey.default_effects(),
        Some(span),
    );
    ctx.store_mutated_local(array, result, mixed_array_ty, Some(span));
}

/// Returns the associative type produced by a string-key write to an indexed array.
fn promoted_assoc_array_type(current_ty: PhpType, value_ty: PhpType) -> PhpType {
    let value_ty = normalize_array_write_element_type(value_ty.codegen_repr());
    let assoc_value_ty = match current_ty.codegen_repr() {
        PhpType::Array(elem_ty) if is_empty_indexed_array_element(elem_ty.as_ref()) => {
            value_ty
        }
        PhpType::Array(elem_ty) => {
            let elem_ty = normalize_array_write_element_type(elem_ty.codegen_repr());
            if elem_ty == value_ty {
                elem_ty
            } else {
                PhpType::Mixed
            }
        }
        _ => PhpType::Mixed,
    };
    PhpType::AssocArray {
        key: Box::new(PhpType::Mixed),
        value: Box::new(assoc_value_ty),
    }
}

/// Lowers a nested array assignment that already carries an expression target.
///
/// A NESTED write (2+ index levels) whose base is a reference-bound LOCAL (`$x = &$arr[0]` then
/// `$x[1][0] = 9`) routes through `lower_nested_ref_bound_local_assign`, which materializes each
/// intermediate as an owned hidden temp (COW-splitting it from the parent slot), writes the leaf
/// value into the innermost temp via the matching in-place set op, then walks back up the chain
/// writing each mutated temp into its parent so the mutation reaches the kind-6 ref cell. Every
/// other shape (plain-local / static-property / instance-property base, or a non-`Heap` container
/// the explicit descent does not handle) stays on the existing generic 2-operand `RuntimeCall`
/// path, unchanged.
fn lower_nested_array_assign(ctx: &mut LoweringContext<'_, '_>, target: &Expr, value: &Expr, span: Span) {
    if let Some((name, chain)) = nested_ref_bound_local_chain(ctx, target) {
        lower_nested_ref_bound_local_assign(ctx, name, &chain, value, span);
        return;
    }
    let target = lower_expr(ctx, target);
    let value = lower_expr(ctx, value);
    ctx.emit_void(
        Op::RuntimeCall,
        vec![target.value, value.value],
        None,
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Walks an `ArrayAccess` chain (`$x[1][0]`, `$x["a"]["b"]`) to its root `ExprKind::Variable`,
/// returning `(name, indices)` when the root local is currently reference-bound (a kind-6 adopted
/// owner whose loads/stores route through `LoadRefCell`/`StoreRefCell`). The indices are collected
/// outermost-first and the chain length is at least 2 (single-level `ArrayAccess{Variable}` routes
/// to `StmtKind::ArrayAssign` before reaching here). Returns `None` for any non-variable root
/// (property / static-property / dynamic-property base) so those stay on the generic path.
fn nested_ref_bound_local_chain<'a>(ctx: &LoweringContext<'_, '_>, target: &'a Expr) -> Option<(&'a str, Vec<&'a Expr>)> {
    let mut indices: Vec<&Expr> = Vec::new();
    let mut node = target;
    loop {
        match &node.kind {
            ExprKind::ArrayAccess { array, index } => {
                indices.push(index);
                node = array;
            }
            ExprKind::Variable(name) => {
                if ctx.is_ref_bound_local(name) {
                    // outermost-first: the indices were collected innermost-first while walking down.
                    indices.reverse();
                    return Some((name, indices));
                }
                return None;
            }
            _ => return None,
        }
    }
}

/// Lowers a NESTED write (2+ index levels) through a reference-bound LOCAL `$name`, mirroring the
/// SLICE-2 `lower_ref_assign_local_nested_append` explicit per-level write-back:
///
/// 1. **Head** — `load_local(name)` emits `LoadRefCell` yielding the alias inner (the aliased
///    array/hash/Mixed box). When the inner is `Heap(Array)` and the first key is a string/Mixed
///    key, `Op::ArrayToHash` de-packs it (recording a write-back obligation).
/// 2. **Descend** — every intermediate level (all but the last index) is read into an owned hidden
///    temp so the subsequent mutation COW-splits the inner away from the parent slot:
///    `Heap(Hash)` → `HashIsset`+`HashGet`+retain (or `HashNew` vivify); `Heap(Array)` → `ArrayGet`;
///    `Heap(Mixed)`/`Heap(Union)` → `__rt_mixed_array_get`.
/// 3. **Leaf** — the value is written into the innermost temp via the matching 3-operand in-place
///    path (`HashSet`/`ArraySet`/`__rt_mixed_array_set`), NOT the 2-operand `lower_mixed_cell_runtime_assign`
///    that loses the fresh-box write.
/// 4. **Per-level write-back** — each mutated temp is written back into its parent at the matching
///    index, walking from the deepest level up to the head, so the mutation reaches the cell.
/// 5. **Tail** — `store_mutated_local(name, head, …)` emits `StoreRefCell` ONLY when the head was
///    relocated (ArrayToHash de-pack, or the first-level read vivified an absent key): in-place
///    mutation leaves the cell pointer unchanged, and a redundant `__rt_ref_cell_store` would
///    decref the still-aliased inner to zero and free it (a use-after-free). Each temp's redundant
///    share is released with the temp's actual type after its write-back (SLICE-2 discipline).
fn lower_nested_ref_bound_local_assign(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    chain: &[&Expr],
    value: &Expr,
    span: Span,
) {
    let depth = chain.len();
    debug_assert!(depth >= 2, "nested ref-bound assign requires 2+ index levels");

    // Lower every index expression once and reuse the SSA value for both the descent read and the
    // ascending write-back (the index is the same key on the way down and up).
    let indices: Vec<LoweredValue> = chain.iter().map(|e| lower_expr(ctx, e)).collect();
    let value_val = lower_expr(ctx, value);

    // 1. Head: load the alias inner through the kind-6 ref cell.
    let mut head = ctx.load_local(name, Some(span));
    let mut head_ty = ctx.builder.value_php_type(head.value);
    let mut needs_storeback = false;

    // De-pack an indexed-array head to a hash when the first key is a string/Mixed key (Zend
    // de-packs on reference-take; mirror `load_local_array_as_hash`). The relocated hash must be
    // stored back to the cell at the tail.
    if head.ir_type == IrType::Heap(IrHeapKind::Array)
        && index_is_boxed_mixed_key(indices[0].ir_type)
    {
        let element_ty = reference_element_type(&head_ty);
        let assoc_ty = promoted_assoc_array_type(head_ty, element_ty);
        let hash = ctx.emit_value(
            Op::ArrayToHash,
            vec![head.value],
            None,
            assoc_ty.clone(),
            Op::ArrayToHash.default_effects(),
            Some(span),
        );
        head = hash;
        head_ty = assoc_ty;
        needs_storeback = true;
    }

    // 2. Descend the intermediate levels (indices 0..depth-2) into owned hidden temps.
    let mut temp_names: Vec<String> = Vec::with_capacity(depth - 1);
    let mut temp_types: Vec<PhpType> = Vec::with_capacity(depth - 1);
    let mut container = head;
    let mut container_ty = head_ty.clone();
    for i in 0..(depth - 1) {
        let key = indices[i];
        let element_ty = reference_element_type(&container_ty);
        let temp_name = ctx.declare_owned_hidden_temp(element_ty.clone());
        let vivified = read_nested_element_into_owned_temp(
            ctx, container, key, &temp_name, element_ty.clone(), span,
        );
        temp_names.push(temp_name);
        temp_types.push(element_ty.clone());
        // A vivified first-level read writes a new hash into the head at index 0, which may
        // relocate the head → the cell must be updated at the tail.
        if i == 0 && vivified {
            needs_storeback = true;
        }
        // The temp becomes the container for the next level. Reload from the temp slot so the
        // write-back up the chain reads the post-mutation pointer (the temp's SSA home is updated
        // by the set op's `store_result_value` when it relocates).
        container = ctx.load_local(temp_names.last().unwrap(), Some(span));
        container_ty = element_ty;
    }

    // 3. Leaf write: write `value` into the innermost temp at the last index.
    let leaf_temp = temp_names[depth - 2].as_str();
    let leaf_container = ctx.load_local(leaf_temp, Some(span));
    write_nested_element_in_place(
        ctx,
        leaf_container,
        temp_types[depth - 2].clone(),
        indices[depth - 1],
        value_val,
        span,
    );

    // 4. Per-level write-back: walk from the deepest temp up to the head, writing each mutated
    //    temp into its parent at the matching index. After each write-back, release the temp's
    //    redundant share (SLICE-2 `:787-803` discipline) ONLY when the parent's set op RETAINS
    //    the value (`HashSet` incref's before storing). `ArraySet` and `__rt_mixed_array_set`
    //    CONSUME the value (transfer ownership without incref), so the temp no longer holds a
    //    live reference — releasing it would double-free the sole remaining share in the parent
    //    slot. In the consumed case, only clear the slot.
    for i in (1..(depth - 1)).rev() {
        let child = ctx.load_local(&temp_names[i], Some(span));
        let parent = ctx.load_local(&temp_names[i - 1], Some(span));
        let parent_ir = parent.ir_type;
        write_nested_element_in_place(
            ctx,
            parent,
            temp_types[i - 1].clone(),
            indices[i],
            child,
            span,
        );
        let consumed = !matches!(parent_ir, IrType::Heap(IrHeapKind::Hash));
        release_owned_hidden_temp(ctx, &temp_names[i], temp_types[i].clone(), consumed, span);
    }
    // Final write-back: temp[0] → head at index[0].
    let child0 = ctx.load_local(&temp_names[0], Some(span));
    let head_ir = head.ir_type;
    write_nested_element_in_place(ctx, head, head_ty.clone(), indices[0], child0, span);
    let head_consumed = !matches!(head_ir, IrType::Heap(IrHeapKind::Hash));
    release_owned_hidden_temp(ctx, &temp_names[0], temp_types[0].clone(), head_consumed, span);

    // 5. Tail: store the (possibly relocated) head back into the ref cell ONLY when a relocation
    //    happened. In-place mutation leaves the cell pointer unchanged; a redundant
    //    `__rt_ref_cell_store` would decref the still-aliased inner to zero and free it.
    if needs_storeback {
        ctx.store_mutated_local(name, head, head_ty, Some(span));
    }
}

/// Reads one nested element into the owned hidden temp `temp_name`, returning whether a vivify
/// branch was taken (an absent hash key auto-vivified to a fresh empty hash).
///
/// - `Heap(Hash)` container → `HashIsset` test; present → `HashGet`+retain stored into the temp;
///   absent → `HashNew` vivify stored into the temp. Both branches branch to a common merge block
///   (SLICE-2 `:733-779` pattern); the temp slot holds the element after merge.
/// - `Heap(Array)` container → `ArrayGet` (integer key coerced) stored into the temp. Assumes the
///   index is in bounds; the (a)-slice test set only writes through existing intermediates.
/// - `Heap(Mixed)`/`Heap(Union)` container → `__rt_mixed_array_get` runtime helper, which boxes
///   typed slots into a fresh owned Mixed cell (the write-back stores it back through the parent).
fn read_nested_element_into_owned_temp(
    ctx: &mut LoweringContext<'_, '_>,
    container: LoweredValue,
    key: LoweredValue,
    temp_name: &str,
    element_ty: PhpType,
    span: Span,
) -> bool {
    match container.ir_type {
        IrType::Heap(IrHeapKind::Hash) => {
            // Mirror SLICE-2 `:733-779`: HashIsset, then present→HashGet+retain / absent→HashNew.
            let exists = ctx.emit_value(
                Op::HashIsset,
                vec![container.value, key.value],
                None,
                PhpType::Bool,
                Op::HashIsset.default_effects(),
                Some(span),
            );
            let split_initialized = ctx.initialized_slots_snapshot();
            let present_block = ctx.builder.create_named_block("nref.present", Vec::new());
            let vivify_block = ctx.builder.create_named_block("nref.vivify", Vec::new());
            let merge = ctx.builder.create_named_block("nref.merge", Vec::new());
            ctx.builder.terminate(Terminator::CondBr {
                cond: exists.value,
                then_target: present_block,
                then_args: Vec::new(),
                else_target: vivify_block,
                else_args: Vec::new(),
            });
            ctx.builder.position_at_end(present_block);
            ctx.restore_initialized_slots(split_initialized.clone());
            let existing = ctx.emit_value(
                Op::HashGet,
                vec![container.value, key.value],
                None,
                element_ty.clone(),
                Op::HashGet.default_effects(),
                Some(span),
            );
            store_value_into_temp(ctx, temp_name, element_ty.clone(), existing, span);
            branch_to(ctx, merge);

            ctx.builder.position_at_end(vivify_block);
            ctx.restore_initialized_slots(split_initialized);
            let vivified = ctx.emit_value(
                Op::HashNew,
                Vec::new(),
                Some(Immediate::Capacity(0)),
                element_ty.clone(),
                Op::HashNew.default_effects(),
                Some(span),
            );
            store_value_into_temp(ctx, temp_name, element_ty, vivified, span);
            branch_to(ctx, merge);

            ctx.builder.position_at_end(merge);
            true
        }
        IrType::Heap(IrHeapKind::Array) => {
            let key_int = coerce_to_int_at_span(ctx, key, Some(span));
            let element = ctx.emit_value(
                Op::ArrayGet,
                vec![container.value, key_int.value],
                None,
                element_ty.clone(),
                Op::ArrayGet.default_effects(),
                Some(span),
            );
            store_value_into_temp(ctx, temp_name, element_ty, element, span);
            false
        }
        IrType::Heap(IrHeapKind::Mixed) | IrType::Heap(IrHeapKind::Union) => {
            let element = ctx.emit_value(
                Op::RuntimeCall,
                vec![container.value, key.value],
                None,
                PhpType::Mixed,
                effects_lookup::runtime_effects(),
                Some(span),
            );
            store_value_into_temp(ctx, temp_name, PhpType::Mixed, element, span);
            false
        }
        _ => {
            // Scalar or unsupported container — the checker loud-errors "Cannot use a scalar value
            // as an array" for reference-bound scalar bases before lowering, so this is a defensive
            // fallback. Produce a Mixed-typed runtime read so the codegen reports a loud unsupported
            // receiver type rather than emitting a Hash/Array op against a non-container operand.
            let element = ctx.emit_value(
                Op::RuntimeCall,
                vec![container.value, key.value],
                None,
                PhpType::Mixed,
                effects_lookup::runtime_effects(),
                Some(span),
            );
            store_value_into_temp(ctx, temp_name, PhpType::Mixed, element, span);
            false
        }
    }
}

/// Writes `value` into `container` at `key` in place via the matching 3-operand set op:
/// `Heap(Hash)` → `HashSet` (or `HashAppend` for a `[]` append — currently unused since the parser
/// lowers `$x[$k][]` to a read+push+writeback sequence); `Heap(Array)` → `ArraySet` (integer key
/// coerced); `Heap(Mixed)`/`Heap(Union)` → `__rt_mixed_array_set` (3-operand `RuntimeCall`). The
/// set op's codegen releases the displaced prior element and updates the container SSA value's
/// home with the possibly-relocated pointer.
fn write_nested_element_in_place(
    ctx: &mut LoweringContext<'_, '_>,
    container: LoweredValue,
    container_ty: PhpType,
    key: LoweredValue,
    value: LoweredValue,
    span: Span,
) {
    match container.ir_type {
        IrType::Heap(IrHeapKind::Hash) => {
            ctx.emit_void(
                Op::HashSet,
                vec![container.value, key.value, value.value],
                None,
                Op::HashSet.default_effects(),
                Some(span),
            );
            release_persisted_string_operand(ctx, key, span);
            release_persisted_string_operand(ctx, value, span);
        }
        IrType::Heap(IrHeapKind::Array) => {
            let key_int = coerce_to_int_at_span(ctx, key, Some(span));
            let value_coerced = coerce_indexed_array_set_value(ctx, &container_ty, value, Some(span));
            ctx.emit_void(
                Op::ArraySet,
                vec![container.value, key_int.value, value_coerced.value],
                None,
                Op::ArraySet.default_effects(),
                Some(span),
            );
            release_persisted_string_operand(ctx, value_coerced, span);
        }
        IrType::Heap(IrHeapKind::Mixed) | IrType::Heap(IrHeapKind::Union) => {
            ctx.emit_void(
                Op::RuntimeCall,
                vec![container.value, key.value, value.value],
                None,
                effects_lookup::runtime_effects(),
                Some(span),
            );
            release_persisted_string_operand(ctx, key, span);
            release_persisted_string_operand(ctx, value, span);
        }
        _ => {
            // Defensive: the checker gates scalar-as-array for reference-bound bases. Fall back to
            // the 2-operand runtime cell assign, which loud-errors at codegen for a scalar receiver.
            ctx.emit_void(
                Op::RuntimeCall,
                vec![container.value, value.value],
                None,
                effects_lookup::runtime_effects(),
                Some(span),
            );
            release_persisted_string_operand(ctx, value, span);
        }
    }
}

/// Releases an owned hidden temp's redundant share with the temp's actual type and clears the
/// backing slot without an additional release (SLICE-2 `:787-803` discipline). After a `HashSet`
/// write-back, the parent slot retains the value (HashSet incref's before storing), so the temp's
/// share is the redundant one and must be decref'd through the matching runtime helper (not via
/// `unset`, which would widen the slot toward `Void`/`Mixed` and decref through the wrong helper,
/// leaking the heap object).
///
/// After an `ArraySet` or 3-operand `__rt_mixed_array_set` write-back, the value is CONSUMED
/// (ownership transferred to the slot without incref), so the temp no longer holds a live
/// reference — releasing it would decref the sole remaining share (the parent slot's) to zero and
/// free it, creating a dangling pointer in the parent. In that case, only clear the slot.
fn release_owned_hidden_temp(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    ty: PhpType,
    consumed: bool,
    span: Span,
) {
    if !consumed {
        let slot = ctx.declare_local(name, ty);
        ctx.release_stored_local_value(name, slot, Some(span));
    }
    ctx.clear_owned_hidden_temp(name, Some(span));
}

/// Lowers `$array[] = value`.
fn lower_array_push(ctx: &mut LoweringContext<'_, '_>, array: &str, value: &Expr, span: Span) {
    let array_value = ctx.load_local(array, Some(span));
    let value = lower_expr(ctx, value);
    let op = if array_value.ir_type == IrType::Heap(crate::ir::IrHeapKind::Array) {
        Op::ArrayPush
    } else if array_value.ir_type == IrType::Heap(crate::ir::IrHeapKind::Mixed) {
        Op::MixedArrayAppend
    } else {
        Op::RuntimeCall
    };
    if op == Op::ArrayPush {
        // A ref-bound local's push must NOT re-store the pushed pointer through
        // `Op::StoreRefCell`: the `ArrayPush` backend already writes the possibly-reallocated
        // array pointer back through the kind-6 cell (`source_load_local_slot` matches the
        // `LoadRefCell` source), so an extra `StoreRefCell` would release the cell's prior
        // inner — the very pointer it is about to store — freeing the live array. Keep the
        // type fact (`updated_ty`) but skip the runtime storeback (`needs_storeback = false`).
        let (array_value, updated_ty, needs_storeback) = if ref_bound_mixed_indexed_array_write(ctx, array, value) {
            (array_value, Some(ctx.local_type(array)), false)
        } else {
            prepare_indexed_array_local_write(ctx, array_value, value, span)
        };
        ctx.emit_void(op, vec![array_value.value, value.value], None, op.default_effects(), Some(span));
        let elem_ty = indexed_array_write_element_type(ctx, array_value, updated_ty.as_ref());
        finish_indexed_array_local_write(ctx, array, array_value, updated_ty, needs_storeback, span);
        release_indexed_array_write_operand(ctx, elem_ty.as_ref(), value, span);
        return;
    }
    ctx.emit_void(op, vec![array_value.value, value.value], None, op.default_effects(), Some(span));
    release_persisted_string_operand(ctx, value, span);
}

/// Prepares an indexed-array local for an offset assignment.
fn prepare_indexed_array_local_set(
    ctx: &mut LoweringContext<'_, '_>,
    array_value: LoweredValue,
    value: LoweredValue,
    span: Span,
) -> (LoweredValue, Option<PhpType>, bool) {
    let current_ty = ctx.builder.value_php_type(array_value.value);
    let value_ty = ctx.builder.value_php_type(value.value);
    if indexed_array_refcounted_set_needs_mixed_conversion(&current_ty, &value_ty) {
        let updated_ty = PhpType::Array(Box::new(PhpType::Mixed));
        let converted = ctx.emit_value(
            Op::ArrayToMixed,
            vec![array_value.value],
            None,
            updated_ty.clone(),
            Op::ArrayToMixed.default_effects(),
            Some(span),
        );
        return (converted, Some(updated_ty), true);
    }
    prepare_indexed_array_local_write(ctx, array_value, value, span)
}

/// Coerces miss-capable scalar reads before writing them into a concrete indexed-array slot.
fn coerce_indexed_array_set_value(
    ctx: &mut LoweringContext<'_, '_>,
    array_ty: &PhpType,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    match array_ty.codegen_repr() {
        PhpType::Array(elem_ty)
            if elem_ty.codegen_repr() == PhpType::Int
                && matches!(
                    ctx.builder.value_php_type(value.value).codegen_repr(),
                    PhpType::Mixed | PhpType::TaggedScalar | PhpType::Union(_)
                ) =>
        {
            coerce_to_int(ctx, value, span)
        }
        _ => value,
    }
}

/// Returns true when a refcounted indexed-array assignment should use Mixed slots.
fn indexed_array_refcounted_set_needs_mixed_conversion(
    current_ty: &PhpType,
    value_ty: &PhpType,
) -> bool {
    let PhpType::Array(elem_ty) = current_ty.codegen_repr() else {
        return false;
    };
    let elem_ty = elem_ty.codegen_repr();
    let value_ty = value_ty.codegen_repr();
    elem_ty != value_ty
        && elem_ty != PhpType::Mixed
        && elem_ty.is_refcounted()
        && value_ty.is_refcounted()
}

/// Converts typed indexed arrays to Mixed when a local write would make them heterogeneous.
pub(super) fn prepare_indexed_array_local_write(
    ctx: &mut LoweringContext<'_, '_>,
    array_value: LoweredValue,
    value: LoweredValue,
    span: Span,
) -> (LoweredValue, Option<PhpType>, bool) {
    let current_ty = ctx.builder.value_php_type(array_value.value);
    let value_ty = ctx.builder.value_php_type(value.value);
    let Some(updated_ty) = indexed_array_write_updated_type(current_ty.clone(), value_ty) else {
        return (array_value, None, false);
    };
    if !indexed_array_write_needs_mixed_conversion(&current_ty, &updated_ty) {
        return (array_value, Some(updated_ty), false);
    }
    let converted = ctx.emit_value(
        Op::ArrayToMixed,
        vec![array_value.value],
        None,
        updated_ty.clone(),
        Op::ArrayToMixed.default_effects(),
        Some(span),
    );
    (converted, Some(updated_ty), true)
}

/// Updates local type facts and emits explicit storeback for converted array writes.
pub(super) fn finish_indexed_array_local_write(
    ctx: &mut LoweringContext<'_, '_>,
    array: &str,
    array_value: LoweredValue,
    updated_ty: Option<PhpType>,
    needs_storeback: bool,
    span: Span,
) {
    let Some(updated_ty) = updated_ty else {
        return;
    };
    if needs_storeback {
        ctx.store_mutated_local(array, array_value, updated_ty, Some(span));
    } else {
        ctx.set_local_type(array, updated_ty);
    }
}

/// Returns true when a ref-bound indexed array should keep its caller-visible element type.
pub(super) fn ref_bound_mixed_indexed_array_write(
    ctx: &LoweringContext<'_, '_>,
    array: &str,
    value: LoweredValue,
) -> bool {
    ctx.is_ref_bound_local(array)
        && matches!(
            ctx.builder.value_php_type(value.value).codegen_repr(),
            PhpType::Mixed | PhpType::Union(_)
        )
}

/// Returns the refined array type after writing a value into an indexed array.
fn indexed_array_write_updated_type(current_ty: PhpType, value_ty: PhpType) -> Option<PhpType> {
    match current_ty.codegen_repr() {
        PhpType::Array(elem_ty) if is_empty_indexed_array_element(elem_ty.as_ref()) => {
            Some(PhpType::Array(Box::new(normalize_empty_array_write_element_type(value_ty))))
        }
        PhpType::Array(elem_ty) if elem_ty.codegen_repr() == PhpType::Mixed => None,
        PhpType::Array(elem_ty) => {
            let elem_ty = elem_ty.codegen_repr();
            if elem_ty == value_ty.codegen_repr() {
                return None;
            }
            let value_ty = normalize_array_write_element_type(value_ty.codegen_repr());
            if elem_ty == value_ty {
                None
            } else {
                Some(PhpType::Array(Box::new(PhpType::Mixed)))
            }
        }
        _ => None,
    }
}

/// Returns true when an indexed-array write needs runtime conversion to Mixed slots.
fn indexed_array_write_needs_mixed_conversion(current_ty: &PhpType, updated_ty: &PhpType) -> bool {
    let PhpType::Array(current_elem) = current_ty.codegen_repr() else {
        return false;
    };
    let PhpType::Array(updated_elem) = updated_ty.codegen_repr() else {
        return false;
    };
    updated_elem.codegen_repr() == PhpType::Mixed
        && current_elem.codegen_repr() != PhpType::Mixed
}

/// Returns true for the placeholder element type used by empty indexed arrays.
fn is_empty_indexed_array_element(elem_ty: &PhpType) -> bool {
    matches!(elem_ty.codegen_repr(), PhpType::Never | PhpType::Void)
}

/// Preserves the first concrete value type written into an empty indexed array.
fn normalize_empty_array_write_element_type(item_type: PhpType) -> PhpType {
    normalize_materialized_element_type(item_type)
}

/// Lowers an assignment with a declared type.
fn lower_typed_assign(
    ctx: &mut LoweringContext<'_, '_>,
    type_expr: &crate::parser::ast::TypeExpr,
    name: &str,
    value: &Expr,
    span: Span,
) {
    let direct_closure = matches!(value.kind, ExprKind::Closure { .. });
    ctx.clear_pending_static_callable_result();
    let php_type = ctx.type_expr_to_php_type_for_value(type_expr);
    let static_callable = static_callable_binding_for_expr(ctx, value);
    let fiber_start_sig = crate::ir_lower::fibers::start_sig_for_expr(ctx, value);
    let callable_array = lower_callable_array_for_assignment(ctx, value, static_callable.as_ref());
    let lowered = callable_array
        .as_ref()
        .map(|assignment| assignment.value)
        .unwrap_or_else(|| lower_expr(ctx, value));
    let lowered = coerce_typed_assign_value(ctx, lowered, &php_type, span);
    ctx.declare_local(name, php_type.clone());
    ctx.store_local(name, lowered, php_type, Some(span));
    let callable_result = if direct_closure {
        ctx.take_pending_static_callable_result()
    } else {
        ctx.clear_pending_static_callable_result();
        None
    };
    let static_callable = callable_array
        .map(|assignment| assignment.target)
        .or(static_callable)
        .or(callable_result);
    if let Some(target) = static_callable {
        ctx.bind_static_callable_local(name, target);
    }
    if let Some(sig) = fiber_start_sig {
        ctx.bind_fiber_start_sig(name, sig);
    }
}

/// Coerces a typed local assignment into the storage shape required by the declared type.
fn coerce_typed_assign_value(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    php_type: &PhpType,
    span: Span,
) -> LoweredValue {
    let target_ty = php_type.codegen_repr();
    let source_ty = ctx.builder.value_php_type(value.value).codegen_repr();
    if source_ty == target_ty {
        return value;
    }
    match target_ty {
        PhpType::Mixed => ctx.emit_value(
            Op::MixedBox,
            vec![value.value],
            None,
            PhpType::Mixed,
            Op::MixedBox.default_effects(),
            Some(span),
        ),
        _ => value,
    }
}

/// Lowers a `foreach` loop using high-level iterator opcodes.
fn lower_foreach(
    ctx: &mut LoweringContext<'_, '_>,
    array: &Expr,
    key_var: Option<&str>,
    value_var: &str,
    value_by_ref: bool,
    body: &[Stmt],
) {
    // Pre-widen locals reassigned inside the loop body (other than the foreach key/value locals,
    // whose per-iteration binding already keeps their type consistent) so later-iteration reads
    // placed before an in-loop reassignment use the widened Mixed slot. See `lower_while`.
    loop_types::prewiden_loop_carried_locals(ctx, &[body], &[], &[]);
    let source = lower_expr(ctx, array);
    let source_php_ty = ctx.builder.value_php_type(source.value);
    let source_ty = source_php_ty.codegen_repr();
    let key_needs_null_init = key_var.is_some_and(|name| !ctx.local_slots.contains_key(name));
    let value_needs_null_init = !ctx.local_slots.contains_key(value_var);
    // A foreach over a concretely-indexed array (`Array` of a non-Mixed element
    // type) always yields integer keys, even though `Op::IterCurrentKey` lowers
    // the key as Mixed. Tag the key local so a `$dst[$key] = ...` write coerces
    // the int-valued Mixed key to int instead of promoting the destination to a
    // hash. Generic `Array(Mixed)`, `AssocArray`, `Mixed`, and `Union` sources
    // may carry string keys and are left untagged so the write promotes.
    if let Some(key_var) = key_var {
        if let PhpType::Array(elem_ty) = &source_php_ty {
            if !matches!(elem_ty.as_ref(), PhpType::Mixed) {
                ctx.mark_foreach_int_key(key_var);
            }
        }
    }
    let iterator = ctx.emit_value(
        Op::IterStart,
        vec![source.value],
        value_by_ref.then_some(Immediate::Bool(true)),
        PhpType::Iterable,
        Op::IterStart.default_effects(),
        Some(array.span),
    );
    if let Some(key_var) = key_var {
        initialize_foreach_mixed_local_if_needed(ctx, key_var, key_needs_null_init, array.span);
    }
    if value_by_ref {
        let value_ty = foreach_ref_value_type(&source_ty);
        ctx.declare_local(value_var, value_ty.clone());
        ctx.set_local_type(value_var, value_ty);
        if !value_needs_null_init {
            ctx.mark_local_initialized(value_var);
            if !ctx.is_ref_bound_local(value_var) {
                ctx.promote_local_ref_cell(value_var, Some(array.span));
            }
        }
    } else {
        let value_ty = foreach_value_type(&source_ty);
        if value_ty == PhpType::Mixed {
            initialize_foreach_mixed_local_if_needed(ctx, value_var, value_needs_null_init, array.span);
        } else if value_needs_null_init {
            ctx.declare_local(value_var, value_ty.clone());
            ctx.set_local_type(value_var, value_ty);
        }
    }
    let header = ctx.builder.create_named_block("foreach.next", Vec::new());
    let body_block = ctx.builder.create_named_block("foreach.body", Vec::new());
    let exit = ctx.builder.create_named_block("foreach.exit", Vec::new());
    branch_to(ctx, header);

    ctx.builder.position_at_end(header);
    let has_next = ctx.emit_value(
        Op::IterNext,
        vec![iterator.value],
        None,
        PhpType::Bool,
        Op::IterNext.default_effects(),
        Some(array.span),
    );
    ctx.builder.terminate(Terminator::CondBr {
        cond: has_next.value,
        then_target: body_block,
        then_args: Vec::new(),
        else_target: exit,
        else_args: Vec::new(),
    });

    ctx.clear_static_callable_locals();
    ctx.builder.position_at_end(body_block);
    let cleanup = ctx
        .value_is_owning_temporary(source)
        .then_some(LoopCleanup { value: source, span: array.span });
    ctx.loop_stack.push(LoopFrame {
        break_block: exit,
        continue_block: header,
        cleanup,
    });
    if let Some(key_var) = key_var {
        let key = ctx.emit_value(
            Op::IterCurrentKey,
            vec![iterator.value],
            None,
            PhpType::Mixed,
            Op::IterCurrentKey.default_effects(),
            Some(array.span),
        );
        ctx.store_local(key_var, key, PhpType::Mixed, Some(array.span));
    }
    if value_by_ref {
        let slot = ctx.declare_local(value_var, foreach_ref_value_type(&source_ty));
        ctx.release_ref_cell_owner(value_var, Some(array.span));
        ctx.emit_void(
            Op::IterCurrentValueRef,
            vec![iterator.value],
            Some(Immediate::LocalSlot(slot)),
            Op::IterCurrentValueRef.default_effects(),
            Some(array.span),
        );
        ctx.mark_ref_bound_local(value_var);
        ctx.mark_local_initialized(value_var);
    } else {
        let value_ty = foreach_value_type(&source_ty);
        let value = ctx.emit_value(
            Op::IterCurrentValue,
            vec![iterator.value],
            None,
            value_ty.clone(),
            Op::IterCurrentValue.default_effects(),
            Some(array.span),
        );
        ctx.store_local(value_var, value, value_ty, Some(array.span));
    }
    lower_block(ctx, body);
    ctx.loop_stack.pop();
    branch_to(ctx, header);
    ctx.builder.position_at_end(exit);
    ctx.clear_static_callable_locals();
    // Release the source when it is a fresh owning temporary (e.g. `foreach
    // (explode(...) as $p)` or a literal array): the iterator borrows it for the
    // duration of the loop, so nothing else frees it once iteration ends. (For an
    // array the iterator aliases the source, so it must NOT be released separately
    // — that would double-free.)
    if ctx.value_is_owning_temporary(source) {
        crate::ir_lower::ownership::release_if_owned(ctx, source, Some(array.span));
    }
}

/// Returns the by-value foreach local type when Phase 04 can keep a concrete element.
fn foreach_value_type(source_ty: &PhpType) -> PhpType {
    match source_ty.codegen_repr() {
        PhpType::Array(elem) if elem.codegen_repr() == PhpType::Callable => PhpType::Callable,
        PhpType::Object(class_name) if class_name == "Phar" || class_name == "PharData" => {
            PhpType::Object("PharFileInfo".to_string())
        }
        _ => PhpType::Mixed,
    }
}

/// Returns the local value type used when a foreach binds the value by reference.
fn foreach_ref_value_type(source_ty: &PhpType) -> PhpType {
    match source_ty.codegen_repr() {
        PhpType::Array(elem) => *elem,
        PhpType::AssocArray { value, .. } => *value,
        _ => PhpType::Mixed,
    }
}

/// Initializes a fresh foreach loop variable to boxed null before the first iteration.
fn initialize_foreach_mixed_local_if_needed(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    needs_init: bool,
    span: Span,
) {
    if !needs_init {
        return;
    }
    // This setup can run once per outer-loop iteration at runtime, overwriting
    // the loop variable. `store_local` owns the carried release: it frees the
    // previous runtime occupant when this synthetic store is loop-carried.
    ctx.declare_local(name, PhpType::Mixed);
    ctx.set_local_type(name, PhpType::Mixed);
    let null = emit_null_value(ctx, Some(span));
    let boxed = ctx.emit_value(
        Op::MixedBox,
        vec![null.value],
        None,
        PhpType::Mixed,
        Op::MixedBox.default_effects(),
        Some(span),
    );
    ctx.store_local(name, boxed, PhpType::Mixed, Some(span));
}

/// Lowers a `switch` with source-ordered pattern evaluation and PHP fallthrough.
fn lower_switch(
    ctx: &mut LoweringContext<'_, '_>,
    subject: &Expr,
    cases: &[(Vec<Expr>, Vec<Stmt>)],
    default: Option<&[Stmt]>,
) {
    let subject = lower_expr(ctx, subject);
    let exit = ctx.builder.create_named_block("switch.exit", Vec::new());
    let default_block = ctx.builder.create_named_block("switch.default", Vec::new());
    let blocks = cases
        .iter()
        .map(|_| ctx.builder.create_named_block("switch.case", Vec::new()))
        .collect::<Vec<_>>();

    // The compact integer jump table is valid only for an integer scrutinee with
    // integer case labels. Any other subject (string, float, mixed) takes the
    // source-ordered dynamic path — see `lower_dynamic_switch_dispatch` for how it
    // picks PHP loose-equality vs the integer fast path per subject/case pair.
    if subject.ir_type == IrType::I64 && can_lower_static_switch(cases) {
        let subject = coerce_to_int(ctx, subject, None);
        lower_static_switch_dispatch(ctx, subject, cases, &blocks, default_block);
    } else {
        lower_dynamic_switch_dispatch(ctx, subject, cases, &blocks, default_block);
    }

    // The state after dispatch (all case conditions evaluated, no body run) is the
    // no-match path's type environment, and it is the environment the code after the
    // switch sees when the subject matched no case. Capturing it here lets us fold the
    // case/default body type mutations back against it so a reassignment that only
    // happens on a case body (especially one that returns) does not leak into the
    // post-switch code where that body never ran.
    let no_match_types = ctx.local_types_snapshot();
    lower_switch_bodies(ctx, cases, default, &blocks, default_block, exit);
    let body_types = ctx.local_types_snapshot();
    let merged = join_local_types(ctx, no_match_types, &body_types);
    ctx.restore_local_types(merged);
}

/// Returns true when every switch case pattern can use the static integer switch terminator.
fn can_lower_static_switch(cases: &[(Vec<Expr>, Vec<Stmt>)]) -> bool {
    cases
        .iter()
        .flat_map(|(case_exprs, _)| case_exprs)
        .all(|case_expr| int_case_value(case_expr).is_some())
}

/// Emits the compact integer-switch dispatch for statically-known case values.
fn lower_static_switch_dispatch(
    ctx: &mut LoweringContext<'_, '_>,
    subject: LoweredValue,
    cases: &[(Vec<Expr>, Vec<Stmt>)],
    blocks: &[BlockId],
    default_block: BlockId,
) {
    let mut switch_cases = Vec::new();
    for ((case_exprs, _), case_block) in cases.iter().zip(blocks) {
        for case_expr in case_exprs {
            let Some(value) = int_case_value(case_expr) else {
                continue;
            };
            switch_cases.push(SwitchCase { value, target: *case_block, args: Vec::new() });
        }
    }
    ctx.builder.terminate(Terminator::Switch {
        scrutinee: subject.value,
        cases: switch_cases,
        default: default_block,
        default_args: Vec::new(),
    });
    ctx.clear_static_callable_locals();
}

/// Emits source-ordered dynamic switch pattern checks for non-literal case expressions.
///
/// PHP `switch` compares the subject against each case with loose equality (`==`).
/// String subjects/labels and float/numeric pairs are dispatched through `Op::LooseEq`
/// so the comparison honors PHP string/numeric coercion rules (`switch (1.5)` matching
/// `case 1.5`, not `case 1`); purely integer-like subject-and-case pairs keep the
/// cheaper `coerce_to_int` + `ICmp` fast path.
fn lower_dynamic_switch_dispatch(
    ctx: &mut LoweringContext<'_, '_>,
    subject: LoweredValue,
    cases: &[(Vec<Expr>, Vec<Stmt>)],
    blocks: &[BlockId],
    default_block: BlockId,
) {
    let subject_is_str = subject.ir_type == IrType::Str;
    // Non-string subjects are coerced to an integer once and reused by the ICmp path.
    let int_subject =
        if subject_is_str { None } else { Some(coerce_to_int(ctx, subject, None)) };
    for ((case_exprs, _), case_block) in cases.iter().zip(blocks) {
        for case_expr in case_exprs {
            let case_value = lower_expr(ctx, case_expr);
            // Strings and floats must use loose equality: coercing a string to int
            // collapses every case to `0 == 0`, and coercing a float to int would
            // truncate the subject (so `switch (1.5) { case 1.5; }` would wrongly
            // match `case 1`). The cheap ICmp fast path stays for integer-like pairs.
            let use_loose_eq = subject_is_str
                || case_value.ir_type == IrType::Str
                || float_loose_eq_pair(subject.ir_type, case_value.ir_type);
            let matched = if use_loose_eq {
                // Loose equality handles string/string, string/scalar, float/numeric,
                // and mixed cases exactly as PHP's `==` would inside an if/elseif chain.
                ctx.emit_value(
                    Op::LooseEq,
                    vec![subject.value, case_value.value],
                    None,
                    PhpType::Bool,
                    Op::LooseEq.default_effects(),
                    Some(case_expr.span),
                )
            } else {
                let case_value = coerce_to_int(ctx, case_value, Some(case_expr.span));
                ctx.emit_value(
                    Op::ICmp,
                    vec![int_subject.expect("non-string subject is pre-coerced").value, case_value.value],
                    Some(Immediate::CmpPredicate(CmpPredicate::Eq)),
                    PhpType::Bool,
                    Op::ICmp.default_effects(),
                    Some(case_expr.span),
                )
            };
            let miss_block = ctx.builder.create_named_block("switch.next", Vec::new());
            ctx.builder.terminate(Terminator::CondBr {
                cond: matched.value,
                then_target: *case_block,
                then_args: Vec::new(),
                else_target: miss_block,
                else_args: Vec::new(),
            });
            ctx.builder.position_at_end(miss_block);
        }
    }
    branch_to(ctx, default_block);
    ctx.clear_static_callable_locals();
}

/// Returns true when a switch subject/case pair must compare via float loose equality:
/// at least one side is a statically-typed float and both are numeric (`int`/`float`).
/// These pairs route through `Op::LooseEq`, which promotes both operands to float, so the
/// subject is not truncated to int (the backend supports float-vs-int loose equality).
///
/// An untyped (`Mixed`) subject holding a float is not covered here: it still takes the
/// integer fast path and truncates, a separate pre-existing loose-equality limitation that
/// needs a tag-aware runtime comparison helper (tracked in issue #397).
fn float_loose_eq_pair(subject_ty: IrType, case_ty: IrType) -> bool {
    let numeric = |ty: IrType| matches!(ty, IrType::I64 | IrType::F64);
    (subject_ty == IrType::F64 || case_ty == IrType::F64) && numeric(subject_ty) && numeric(case_ty)
}

/// Lowers switch case/default bodies and preserves PHP fallthrough between adjacent bodies.
fn lower_switch_bodies(
    ctx: &mut LoweringContext<'_, '_>,
    cases: &[(Vec<Expr>, Vec<Stmt>)],
    default: Option<&[Stmt]>,
    blocks: &[BlockId],
    default_block: BlockId,
    exit: BlockId,
) {
    ctx.clear_static_callable_locals();
    ctx.loop_stack.push(LoopFrame {
        break_block: exit,
        continue_block: exit,
        cleanup: None,
    });
    for (index, ((_, body), block)) in cases.iter().zip(blocks).enumerate() {
        ctx.builder.position_at_end(*block);
        lower_block(ctx, body);
        if !ctx.builder.insertion_block_is_terminated() {
            if let Some(next_block) = blocks.get(index + 1) {
                branch_to(ctx, *next_block);
            } else {
                branch_to(ctx, default_block);
            }
        }
        ctx.clear_static_callable_locals();
    }
    ctx.builder.position_at_end(default_block);
    if let Some(default) = default {
        lower_block(ctx, default);
    }
    if !ctx.builder.insertion_block_is_terminated() {
        branch_to(ctx, exit);
    }
    ctx.loop_stack.pop();
    ctx.builder.position_at_end(exit);
    ctx.clear_static_callable_locals();
}

/// Lowers include/require statements through a high-level runtime call.
fn lower_include(ctx: &mut LoweringContext<'_, '_>, path: &Expr, once: bool, required: bool, span: Span) {
    let path = lower_expr(ctx, path);
    let label = format!("include once={} required={}", once, required);
    let data = ctx.intern_string(&label);
    ctx.emit_void(
        Op::RuntimeCall,
        vec![path.value],
        Some(Immediate::Data(data)),
        effects_lookup::runtime_effects(),
        Some(span),
    );
    ctx.clear_static_callable_locals();
}

/// Lowers an include-once marker.
fn lower_include_once_mark(ctx: &mut LoweringContext<'_, '_>, label: &str, span: Span) {
    let data = ctx.intern_string(label);
    ctx.emit_void(
        Op::IncludeOnceMark,
        Vec::new(),
        Some(Immediate::Data(data)),
        Op::IncludeOnceMark.default_effects(),
        Some(span),
    );
}

/// Lowers an include-once guarded body.
fn lower_include_once_guard(ctx: &mut LoweringContext<'_, '_>, label: &str, body: &[Stmt], span: Span) {
    let data = ctx.intern_string(label);
    let should_run = ctx
        .builder
        .emit_with_effects(
            Op::IncludeOnceGuard,
            Vec::new(),
            Some(Immediate::Data(data)),
            IrType::I64,
            PhpType::Bool,
            Ownership::NonHeap,
            Op::IncludeOnceGuard.default_effects(),
            Some(span),
        )
        .expect("include_once_guard produces a branch condition");
    let body_block = ctx.builder.create_named_block("include_once_body", Vec::new());
    let after_block = ctx.builder.create_named_block("include_once_after", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: should_run,
        then_target: body_block,
        then_args: Vec::new(),
        else_target: after_block,
        else_args: Vec::new(),
    });
    ctx.clear_static_callable_locals();
    ctx.builder.position_at_end(body_block);
    lower_block(ctx, body);
    branch_to(ctx, after_block);
    ctx.builder.position_at_end(after_block);
    ctx.clear_static_callable_locals();
}

/// Lowers a throwing statement into a terminator.
fn lower_throw(ctx: &mut LoweringContext<'_, '_>, expr: &Expr) {
    let value = lower_expr(ctx, expr);
    terminate_throw(ctx, value.value);
}

/// Lowers a `try`/`catch` statement into a runtime handler and explicit catch-dispatch blocks.
fn lower_try(
    ctx: &mut LoweringContext<'_, '_>,
    try_body: &[Stmt],
    catches: &[CatchClause],
    finally_body: Option<&[Stmt]>,
    span: Span,
) {
    if let Some(finally_body) = finally_body {
        lower_try_with_finally(ctx, try_body, catches, finally_body, span);
        return;
    }

    lower_try_catch(ctx, try_body, catches, span);
}

/// Lowers a `try`/`catch` statement without a `finally` block.
fn lower_try_catch(
    ctx: &mut LoweringContext<'_, '_>,
    try_body: &[Stmt],
    catches: &[CatchClause],
    span: Span,
) {
    let handler_block = ctx.builder.create_named_block("try.catch_dispatch", Vec::new());
    let after_block = ctx.builder.create_named_block("try.after", Vec::new());
    let handler_token = handler_block.as_raw() as i64;

    ctx.clear_static_callable_locals();
    ctx.emit_void(
        Op::TryPushHandler,
        Vec::new(),
        Some(Immediate::I64(handler_token)),
        Op::TryPushHandler.default_effects(),
        Some(span),
    );
    lower_block(ctx, try_body);
    if !ctx.builder.insertion_block_is_terminated() {
        emit_try_pop_handler(ctx, handler_token, span);
        branch_to(ctx, after_block);
    }

    ctx.builder.position_at_end(handler_block);
    emit_try_pop_handler(ctx, handler_token, span);
    lower_catch_dispatch(ctx, catches, after_block, span);
    ctx.builder.position_at_end(after_block);
    ctx.clear_static_callable_locals();
}

/// Lowers `try`/`catch`/`finally` using duplicated finalizer bodies for explicit exits.
fn lower_try_with_finally(
    ctx: &mut LoweringContext<'_, '_>,
    try_body: &[Stmt],
    catches: &[CatchClause],
    finally_body: &[Stmt],
    span: Span,
) {
    if catches.is_empty() {
        lower_try_finally_without_catches(ctx, try_body, finally_body);
    } else {
        lower_try_catch_finally(ctx, try_body, catches, finally_body, span);
    }
}

/// Lowers a `try`/`finally` statement with no catch clauses.
fn lower_try_finally_without_catches(
    ctx: &mut LoweringContext<'_, '_>,
    try_body: &[Stmt],
    finally_body: &[Stmt],
) {
    let depth = push_finally_frame(ctx, finally_body, true, None);
    lower_block(ctx, try_body);
    pop_finally_frame_if_active(ctx, depth);
    if !ctx.builder.insertion_block_is_terminated() {
        lower_block(ctx, finally_body);
    }
}

/// Lowers a `try`/`catch`/`finally` statement while preserving catch-before-finally order.
fn lower_try_catch_finally(
    ctx: &mut LoweringContext<'_, '_>,
    try_body: &[Stmt],
    catches: &[CatchClause],
    finally_body: &[Stmt],
    span: Span,
) {
    let handler_block = ctx.builder.create_named_block("try.catch_dispatch", Vec::new());
    let after_block = ctx.builder.create_named_block("try.after", Vec::new());
    let handler_token = handler_block.as_raw() as i64;

    ctx.clear_static_callable_locals();
    ctx.emit_void(
        Op::TryPushHandler,
        Vec::new(),
        Some(Immediate::I64(handler_token)),
        Op::TryPushHandler.default_effects(),
        Some(span),
    );
    let depth = push_finally_frame(ctx, finally_body, false, Some((handler_token, span)));
    lower_block(ctx, try_body);
    pop_finally_frame_if_active(ctx, depth);
    if !ctx.builder.insertion_block_is_terminated() {
        emit_try_pop_handler(ctx, handler_token, span);
        lower_block(ctx, finally_body);
        branch_to(ctx, after_block);
    }

    ctx.builder.position_at_end(handler_block);
    emit_try_pop_handler(ctx, handler_token, span);
    lower_catch_dispatch_with_finally(ctx, catches, after_block, finally_body, span);
    ctx.builder.position_at_end(after_block);
    ctx.clear_static_callable_locals();
}

/// Emits the runtime cleanup for a pushed try/catch handler.
fn emit_try_pop_handler(ctx: &mut LoweringContext<'_, '_>, handler_token: i64, span: Span) {
    ctx.emit_void(
        Op::TryPopHandler,
        Vec::new(),
        Some(Immediate::I64(handler_token)),
        Op::TryPopHandler.default_effects(),
        Some(span),
    );
}

/// Lowers ordered catch matching from the current exception handler block.
fn lower_catch_dispatch(
    ctx: &mut LoweringContext<'_, '_>,
    catches: &[CatchClause],
    after_block: BlockId,
    span: Span,
) {
    for catch in catches {
        let catch_body = ctx.builder.create_named_block("try.catch_body", Vec::new());
        let next_catch = ctx.builder.create_named_block("try.catch_next", Vec::new());
        lower_catch_match(ctx, catch, catch_body, next_catch, span);
        ctx.builder.position_at_end(catch_body);
        lower_catch_bind(ctx, catch, span);
        lower_block(ctx, &catch.body);
        if !ctx.builder.insertion_block_is_terminated() {
            branch_to(ctx, after_block);
        }
        ctx.clear_static_callable_locals();
        ctx.builder.position_at_end(next_catch);
    }

    let current = lower_current_exception(ctx, span);
    ctx.builder.terminate(Terminator::Throw { value: current.value });
}

/// Lowers catch dispatch for `try`/`catch`/`finally`.
fn lower_catch_dispatch_with_finally(
    ctx: &mut LoweringContext<'_, '_>,
    catches: &[CatchClause],
    after_block: BlockId,
    finally_body: &[Stmt],
    span: Span,
) {
    for catch in catches {
        let catch_body = ctx.builder.create_named_block("try.catch_body", Vec::new());
        let next_catch = ctx.builder.create_named_block("try.catch_next", Vec::new());
        lower_catch_match(ctx, catch, catch_body, next_catch, span);
        ctx.builder.position_at_end(catch_body);
        lower_catch_bind(ctx, catch, span);
        let depth = push_finally_frame(ctx, finally_body, true, None);
        lower_block(ctx, &catch.body);
        pop_finally_frame_if_active(ctx, depth);
        if !ctx.builder.insertion_block_is_terminated() {
            lower_block(ctx, finally_body);
            branch_to(ctx, after_block);
        }
        ctx.clear_static_callable_locals();
        ctx.builder.position_at_end(next_catch);
    }

    let current = lower_current_exception(ctx, span);
    lower_block(ctx, finally_body);
    if !ctx.builder.insertion_block_is_terminated() {
        ctx.builder.terminate(Terminator::Throw { value: current.value });
    }
}

/// Emits the match tests for one catch clause and branches to body or next clause.
fn lower_catch_match(
    ctx: &mut LoweringContext<'_, '_>,
    catch: &CatchClause,
    catch_body: BlockId,
    next_catch: BlockId,
    span: Span,
) {
    if catch.exception_types.is_empty() {
        branch_to(ctx, next_catch);
        return;
    }

    for (idx, catch_type) in catch.exception_types.iter().enumerate() {
        let mismatch = if idx + 1 == catch.exception_types.len() {
            next_catch
        } else {
            ctx.builder.create_named_block("try.catch_type_next", Vec::new())
        };
        let current = lower_current_exception(ctx, span);
        let data = ctx.intern_class_name(catch_type.as_str());
        let matched = ctx.emit_value(
            Op::InstanceOf,
            vec![current.value],
            Some(Immediate::Data(data)),
            PhpType::Bool,
            Op::InstanceOf.default_effects(),
            Some(span),
        );
        ctx.builder.terminate(Terminator::CondBr {
            cond: matched.value,
            then_target: catch_body,
            then_args: Vec::new(),
            else_target: mismatch,
            else_args: Vec::new(),
        });
        if idx + 1 != catch.exception_types.len() {
            ctx.builder.position_at_end(mismatch);
        }
    }
}

/// Emits the current exception value as an object-typed SSA value.
fn lower_current_exception(ctx: &mut LoweringContext<'_, '_>, span: Span) -> LoweredValue {
    ctx.emit_value(
        Op::CatchCurrent,
        Vec::new(),
        None,
        PhpType::Object("Throwable".to_string()),
        Op::CatchCurrent.default_effects(),
        Some(span),
    )
}

/// Binds and clears the active exception for a matched catch clause.
fn lower_catch_bind(ctx: &mut LoweringContext<'_, '_>, catch: &CatchClause, span: Span) {
    let (immediate, php_type) = catch.variable.as_ref().map_or((None, PhpType::Void), |variable| {
        let php_type = catch_variable_type(catch);
        let slot = ctx.declare_local(variable, php_type.clone());
        ctx.set_local_type(variable, php_type.clone());
        (Some(Immediate::LocalSlot(slot)), php_type)
    });
    ctx.builder.emit_with_effects(
        Op::CatchBind,
        Vec::new(),
        immediate,
        IrType::Void,
        php_type,
        Ownership::NonHeap,
        Op::CatchBind.default_effects(),
        Some(span),
    );
}

/// Returns the local type to use for a catch variable.
fn catch_variable_type(catch: &CatchClause) -> PhpType {
    if catch.exception_types.len() == 1 {
        return PhpType::Object(catch.exception_types[0].trim_start_matches('\\').to_string());
    }
    PhpType::Object("Throwable".to_string())
}

/// Lowers a `break` terminator.
fn lower_break(ctx: &mut LoweringContext<'_, '_>, level: usize) {
    let Some(frame) = loop_target(ctx, level) else {
        ctx.builder.terminate(Terminator::Unreachable);
        return;
    };
    terminate_branch(ctx, frame.break_block, loop_cleanup_count_for_branch(level));
}

/// Lowers a `continue` terminator.
fn lower_continue(ctx: &mut LoweringContext<'_, '_>, level: usize) {
    let Some(frame) = loop_target(ctx, level) else {
        ctx.builder.terminate(Terminator::Unreachable);
        return;
    };
    terminate_branch(ctx, frame.continue_block, loop_cleanup_count_for_branch(level));
}

/// Lowers `goto label;` as an unconditional branch to the label's block.
///
/// The target block is shared with the matching `label:` (created lazily by whichever is lowered
/// first), so forward and backward jumps both resolve to one block. Routed through
/// `terminate_branch` so the jump runs any pending `finally` bodies exactly as `break`/`continue`
/// do. PHP variables live in memory slots, not SSA block arguments, so the branch needs no args:
/// the label block reloads them from the same slots.
fn lower_goto(ctx: &mut LoweringContext<'_, '_>, label: &str) {
    let target = ctx.label_block(label);
    // A `goto` keeps the current loop frames live: it never closes an enclosing loop the way a
    // `break`/`continue` does, so no innermost loop cleanups are emitted here (count `0`). Pending
    // `finally` bodies are still run by `terminate_branch` itself.
    terminate_branch(ctx, target, 0);
}

/// Lowers a `label:` marker by closing the current straight-line block with a fall-through branch
/// into the label's (shared) block and continuing emission there.
///
/// Uses `branch_to` rather than `terminate_branch`: falling into a label does not leave any
/// enclosing `try`, so no `finally` runs. If control reaching the label is already terminated
/// (e.g. the preceding statement was a `return`/`goto`), `branch_to` is a no-op and emission simply
/// resumes in the label block, which remains reachable through any `goto` that targets it.
fn lower_label(ctx: &mut LoweringContext<'_, '_>, label: &str) {
    let target = ctx.label_block(label);
    branch_to(ctx, target);
    ctx.builder.position_at_end(target);
}

/// Lowers a return statement using the current function return contract.
fn lower_return(ctx: &mut LoweringContext<'_, '_>, value_expr: Option<&Expr>, span: Span) {
    // A by-reference-returning function hands the caller the ref-cell pointer of the
    // returned property (`function &f() { return $obj->prop; }`), so `$x = &f()` aliases
    // it. The cell pointer is materialized as the declared return type so the ABI return
    // convention matches the caller's expectation for pointer-sized property types.
    if ctx.by_ref_return {
        if let Some(Expr { kind: ExprKind::PropertyAccess { object, property }, .. }) = value_expr {
            let object = lower_expr(ctx, object);
            let data = ctx.intern_string(property);
            let result_ty = ctx.return_php_type.clone();
            let cell_ptr = ctx.emit_value(
                Op::LoadPropRefCell,
                vec![object.value],
                Some(Immediate::Data(data)),
                result_ty,
                Op::LoadPropRefCell.default_effects(),
                Some(span),
            );
            terminate_return(ctx, Some(cell_ptr.value));
            return;
        }
    }
    if ctx.return_type == IrType::Void {
        if let Some(value_expr) = value_expr {
            lower_expr(ctx, value_expr);
        }
        terminate_return(ctx, None);
        return;
    }
    let value = if let Some(value_expr) = value_expr {
        lower_expr(ctx, value_expr)
    } else {
        emit_null_value(ctx, Some(span))
    };
    let value = reload_returned_assignment_local(ctx, value_expr, value, span);
    let value = return_type_guard::emit_checked_downcast_return_guard(ctx, value, span);
    let value = coerce_to_return_type(ctx, value, Some(span));
    let value = acquire_borrowed_return_value(ctx, value, span);
    let value = acquire_returned_this(ctx, value_expr, value, span);
    let value = persist_scratch_return_string(ctx, value, span);
    terminate_return(ctx, Some(value.value));
}

/// Rebalances `return $x = <expr>;` (including the by-reference array-literal desugar,
/// whose yield is `$hidden_yield = $hidden_temp`) by re-loading the just-stored local.
///
/// The plain-local store path acquires the stored value once and hands that same acquire
/// back as the assignment's yield, so a direct `return` of the yield claims a reference
/// the target's slot already owns: the epilogue cleanup of the slot then consumes the
/// caller's share and the caller reads freed memory (silent use-after-free). Returning a
/// fresh `LoadLocal` of the target instead routes the shape through the existing
/// returned-slot ownership transfer (`direct_return_local_slots` excludes the slot from
/// epilogue cleanup), so exactly one owner — the caller — remains. Applies only when the
/// yield is the store's own `Acquire` on a refcounted plain-slot local; every other yield
/// shape (owned temps, ref-bound or global targets, by-ref returns, `result_target`
/// compound reads) keeps its current ownership behavior.
fn reload_returned_assignment_local(
    ctx: &mut LoweringContext<'_, '_>,
    value_expr: Option<&Expr>,
    value: LoweredValue,
    span: Span,
) -> LoweredValue {
    if ctx.by_ref_return {
        return value;
    }
    let Some(Expr {
        kind:
            ExprKind::Assignment {
                target,
                result_target: None,
                ..
            },
        ..
    }) = value_expr
    else {
        return value;
    };
    let ExprKind::Variable(name) = &target.kind else {
        return value;
    };
    if !ctx.local_uses_plain_slot_storage(name) {
        return value;
    }
    if !ctx.builder.value_php_type(value.value).codegen_repr().is_refcounted() {
        return value;
    }
    // The double-claim only exists when the yield IS the store's acquire; any other
    // defining op means the store transferred or skipped the retain and stays balanced.
    if ctx.builder.value_defining_op(value.value) != Some(Op::Acquire) {
        return value;
    }
    ctx.load_local(name, Some(span))
}

/// Acquires the receiver when a method does `return $this`.
///
/// `$this` is a borrowed reference to the receiver the caller still owns. A return
/// value is handed to the caller as owned, so without an extra reference the
/// caller's release of the (often discarded, as in fluent `$obj->setX(...)->setY()`)
/// result drops the object's refcount to zero and runs its destructor while the
/// original binding is still live — a use-after-free for any class with a
/// destructor. Incrementing the refcount here balances that release.
fn acquire_returned_this(
    ctx: &mut LoweringContext<'_, '_>,
    value_expr: Option<&Expr>,
    value: LoweredValue,
    span: Span,
) -> LoweredValue {
    if !matches!(value_expr.map(|expr| &expr.kind), Some(ExprKind::This)) {
        return value;
    }
    crate::ir_lower::ownership::acquire_if_refcounted(ctx, value, Some(span))
}

/// Copies scratch-backed string results before they cross a function boundary.
fn persist_scratch_return_string(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) -> LoweredValue {
    if value.ir_type != IrType::Str {
        return value;
    }
    let Some(op) = ctx.builder.value_defining_op(value.value) else {
        return value;
    };
    if !string_op_uses_scratch_storage(op) {
        return value;
    }
    ctx.emit_value(
        Op::StrPersist,
        vec![value.value],
        None,
        PhpType::Str,
        Op::StrPersist.default_effects(),
        Some(span),
    )
}

/// Acquires return values read from heap containers before local cleanup runs.
fn acquire_borrowed_return_value(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) -> LoweredValue {
    if ctx.value_is_owning_temporary(value) {
        return value;
    }
    let php_type = ctx.builder.value_php_type(value.value);
    if !Ownership::php_type_needs_lifetime_tracking(&php_type) {
        return value;
    }
    if !matches!(
        ctx.builder.value_defining_op(value.value),
        Some(
            Op::ArrayGet
                | Op::HashGet
                | Op::PropGet
                | Op::DynamicPropGet
                | Op::NullsafePropGet
        )
    ) {
        return value;
    }
    crate::ir_lower::ownership::acquire_if_refcounted(ctx, value, Some(span))
}

/// Terminates with a return after running active finally bodies from inner to outer.
fn terminate_return(ctx: &mut LoweringContext<'_, '_>, value: Option<crate::ir::ValueId>) {
    if run_innermost_finally(ctx, false) {
        if !ctx.builder.insertion_block_is_terminated() {
            terminate_return(ctx, value);
        }
        return;
    }
    emit_innermost_loop_cleanups(ctx, ctx.loop_stack.len());
    ctx.builder.terminate(Terminator::Return { value });
}

/// Terminates with a branch after running active finally bodies from inner to outer.
fn terminate_branch(ctx: &mut LoweringContext<'_, '_>, target: BlockId, loop_cleanup_count: usize) {
    if run_innermost_finally(ctx, false) {
        if !ctx.builder.insertion_block_is_terminated() {
            terminate_branch(ctx, target, loop_cleanup_count);
        }
        return;
    }
    emit_innermost_loop_cleanups(ctx, loop_cleanup_count);
    ctx.builder.terminate(Terminator::Br { target, args: Vec::new() });
}

/// Terminates with a throw after running finally bodies that apply to uncaught throws.
fn terminate_throw(ctx: &mut LoweringContext<'_, '_>, value: crate::ir::ValueId) {
    if run_innermost_finally(ctx, true) {
        if !ctx.builder.insertion_block_is_terminated() {
            terminate_throw(ctx, value);
        }
        return;
    }
    emit_innermost_loop_cleanups(ctx, ctx.loop_stack.len());
    ctx.builder.terminate(Terminator::Throw { value });
}

/// Returns how many inner loop cleanups a multi-level branch skips.
fn loop_cleanup_count_for_branch(level: usize) -> usize {
    level.max(1).saturating_sub(1)
}

/// Emits cleanup for the innermost active loops that will not reach their exit block.
fn emit_innermost_loop_cleanups(ctx: &mut LoweringContext<'_, '_>, count: usize) {
    let frames = ctx
        .loop_stack
        .iter()
        .rev()
        .take(count)
        .copied()
        .collect::<Vec<_>>();
    for frame in frames {
        if let Some(cleanup) = frame.cleanup {
            crate::ir_lower::ownership::release_if_owned(ctx, cleanup.value, Some(cleanup.span));
        }
    }
}

/// Runs and removes the innermost applicable finally frame.
fn run_innermost_finally(ctx: &mut LoweringContext<'_, '_>, is_throw: bool) -> bool {
    let Some(frame) = ctx.finally_stack.last() else {
        return false;
    };
    if is_throw && !frame.run_on_throw {
        return false;
    }
    let frame = ctx
        .finally_stack
        .pop()
        .expect("finally frame disappeared after last() check");
    if let Some((handler_token, span)) = frame.handler_cleanup {
        emit_try_pop_handler(ctx, handler_token, span);
    }
    lower_block(ctx, &frame.body);
    true
}

/// Pushes a finalizer and returns the stack depth before the push.
fn push_finally_frame(
    ctx: &mut LoweringContext<'_, '_>,
    body: &[Stmt],
    run_on_throw: bool,
    handler_cleanup: Option<(i64, Span)>,
) -> usize {
    let depth = ctx.finally_stack.len();
    ctx.finally_stack.push(FinallyFrame {
        body: body.to_vec(),
        run_on_throw,
        handler_cleanup,
    });
    depth
}

/// Removes a finalizer when the protected body fell through normally.
fn pop_finally_frame_if_active(ctx: &mut LoweringContext<'_, '_>, depth: usize) {
    if ctx.finally_stack.len() > depth {
        ctx.finally_stack.pop();
    }
}

/// Lowers a global constant declaration.
fn lower_const_decl(ctx: &mut LoweringContext<'_, '_>, name: &str, value: &Expr, span: Span) {
    let value = lower_expr(ctx, value);
    let data = ctx.intern_global_name(name);
    ctx.emit_void(
        Op::StoreGlobal,
        vec![value.value],
        Some(Immediate::GlobalName(data)),
        Op::StoreGlobal.default_effects(),
        Some(span),
    );
}

/// Lowers simple positional list destructuring into indexed reads plus local writes.
fn lower_list_unpack(ctx: &mut LoweringContext<'_, '_>, vars: &[String], value: &Expr, span: Span) {
    let source = lower_expr(ctx, value);
    let item_type = list_unpack_item_type(ctx, source.value);
    let get_op = list_unpack_get_op(source.ir_type);
    for (index, var) in vars.iter().enumerate() {
        let index_value = lower_list_unpack_index(ctx, index, span);
        let item = ctx.emit_value(
            get_op,
            vec![source.value, index_value.value],
            None,
            item_type.clone(),
            get_op.default_effects(),
            Some(span),
        );
        ctx.store_local(var, item, item_type.clone(), Some(span));
    }
}

/// Emits the positional integer key used to read one list-unpack element.
pub(super) fn lower_list_unpack_index(ctx: &mut LoweringContext<'_, '_>, index: usize, span: Span) -> LoweredValue {
    ctx.emit_value(
        Op::ConstI64,
        Vec::new(),
        Some(Immediate::I64(index as i64)),
        PhpType::Int,
        Op::ConstI64.default_effects(),
        Some(span),
    )
}

/// Returns the element-read opcode for a list-unpack source value.
pub(super) fn list_unpack_get_op(source_type: IrType) -> Op {
    match source_type {
        IrType::Heap(crate::ir::IrHeapKind::Array) => Op::ArrayGet,
        IrType::Heap(crate::ir::IrHeapKind::Hash) => Op::HashGet,
        _ => Op::RuntimeCall,
    }
}

/// Returns the PHP type assigned to each simple list-unpack destination.
pub(super) fn list_unpack_item_type(ctx: &LoweringContext<'_, '_>, source: crate::ir::ValueId) -> PhpType {
    let item_type = match ctx.builder.value_php_type(source).codegen_repr() {
        PhpType::Array(elem_ty) => *elem_ty,
        PhpType::AssocArray { value, .. } => *value,
        _ => PhpType::Mixed,
    };
    normalize_materialized_element_type(item_type)
}

/// Normalizes non-materializable element metadata to the null sentinel.
fn normalize_materialized_element_type(item_type: PhpType) -> PhpType {
    match item_type {
        PhpType::Never => PhpType::Void,
        other => other,
    }
}

/// Normalizes indexed-array write payloads to storage shapes Phase 04 can lower.
fn normalize_array_write_element_type(item_type: PhpType) -> PhpType {
    let item_type = normalize_materialized_element_type(item_type);
    if item_type.is_refcounted() && !matches!(item_type, PhpType::Str) {
        PhpType::Mixed
    } else {
        item_type
    }
}

/// Declares global aliases in the local slot table.
fn lower_global(ctx: &mut LoweringContext<'_, '_>, vars: &[String]) {
    for var in vars {
        let php_type = ctx.global_alias_type(var);
        ctx.declare_local_with_kind(var, php_type, LocalKind::GlobalAlias);
    }
}

/// Lowers a static local variable initialization behind a whole-initializer once-guard.
///
/// Called both for a direct `StmtKind::StaticVar` statement and, with `init` substituted for the
/// coalesced default, from `lower_block`'s `static $x; $x ??= <default>;` fold (see
/// `fold_static_null_coalesce_pair`).
///
/// Emits `check flag → CondBr(already-initialized ? skip : eval)`, then lowers `init` (and the
/// commit `Op::InitStaticLocal`) INSIDE the `eval` block, merging back at a shared `after` block.
/// This is the fix for the gap `fold_static_null_coalesce_pair`'s doc comment describes:
/// previously `init`'s value-producing instructions sat straight-line BEFORE `Op::InitStaticLocal`
/// in the same block, so they ran unconditionally on every call even though only the first call's
/// result was ever stored. Wrapping `init`'s lowering itself in the guarded block means a
/// side-effecting or heap-allocating `<init>` now runs exactly once, matching PHP.
///
/// The slot is declared with a placeholder `Void` storage type BEFORE `init` is lowered (so the
/// once-flag check can name it), then corrected to `init`'s actual lowered type via
/// `set_local_type_exact` once known — mirroring how the direct-null-init case already leaves a
/// `Void`-typed slot for a later `??=` to widen. This is codegen-safe because
/// `crate::codegen_ir::lower_inst::static_locals::resolve_static_local_slot` reads the local's
/// FINAL committed type from the module (after all of `ir_lower` has run), not whatever type was
/// current mid-lowering.
fn lower_static_var(ctx: &mut LoweringContext<'_, '_>, name: &str, init: &Expr, span: Span) {
    let slot = ctx.declare_local_with_kind(name, PhpType::Void, LocalKind::StaticLocal);
    let is_initialized = ctx.emit_value(
        Op::StaticLocalInitialized,
        Vec::new(),
        Some(Immediate::LocalSlot(slot)),
        PhpType::Bool,
        Op::StaticLocalInitialized.default_effects(),
        Some(span),
    );

    let eval_block = ctx.builder.create_named_block("static_local_init.eval", Vec::new());
    let after_block = ctx.builder.create_named_block("static_local_init.after", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: is_initialized.value,
        then_target: after_block,
        then_args: Vec::new(),
        else_target: eval_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(eval_block);
    let value = lower_expr(ctx, init);
    ctx.set_local_type_exact(name, ctx.builder.value_php_type(value.value));
    ctx.builder.emit_with_effects(
        Op::InitStaticLocal,
        vec![value.value],
        Some(Immediate::LocalSlot(slot)),
        IrType::Void,
        PhpType::Void,
        Ownership::NonHeap,
        Op::InitStaticLocal.default_effects(),
        Some(span),
    );
    branch_to(ctx, after_block);

    ctx.builder.position_at_end(after_block);
}

/// Lowers an object property write.
fn lower_property_assign(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    value: &Expr,
    span: Span,
) {
    let object = lower_expr(ctx, object);
    let value_expr = value;
    let lowered_value = lower_expr(ctx, value_expr);
    let value = contextualize_property_array_assignment(
        ctx,
        object.value,
        property,
        lowered_value,
        value_expr,
        span,
    );
    if magic_set_receiver_has_method(ctx, object.value, property) {
        lower_magic_property_set(ctx, object.value, property, value, span);
        return;
    }
    // Route a write to a set-hooked property to its `__propset_<p>($value)` accessor, except inside
    // that property's own accessor where `$this->prop = v` must write the raw backing slot.
    if set_hook_receiver_has_accessor(ctx, object.value, property)
        && !ctx.in_own_property_accessor(property)
    {
        lower_property_hook_set(ctx, object.value, property, value, span);
        return;
    }
    let data = ctx.intern_string(property);
    ctx.emit_void(
        Op::PropSet,
        vec![object.value, value.value],
        Some(Immediate::Data(data)),
        Op::PropSet.default_effects(),
        Some(span),
    );
    if let Some(property_ty) = object_property_type(ctx, object.value, property) {
        release_property_assignment_source_after_retaining_store(ctx, &property_ty, value, span);
    }
}

/// Returns true when a property write should dispatch to `__set`.
fn magic_set_receiver_has_method(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
) -> bool {
    let PhpType::Object(class_name) = ctx.builder.value_php_type(object).codegen_repr() else {
        return false;
    };
    let normalized = class_name.trim_start_matches('\\');
    let Some(class_info) = ctx.classes.get(normalized) else {
        return false;
    };
    if class_info.properties.iter().any(|(name, _)| name == property) {
        return false;
    }
    class_info
        .methods
        .contains_key(&php_symbol_key("__set"))
}

/// Lowers an undeclared property write to a normal `__set` instance-method call.
fn lower_magic_property_set(
    ctx: &mut LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
    value: LoweredValue,
    span: Span,
) {
    let property_data = ctx.intern_string(property);
    let property_name = ctx.emit_value(
        Op::ConstStr,
        Vec::new(),
        Some(Immediate::Data(property_data)),
        PhpType::Str,
        Op::ConstStr.default_effects(),
        Some(span),
    );
    let method_data = ctx.intern_string("__set");
    ctx.emit_void(
        Op::MethodCall,
        vec![object, property_name.value, value.value],
        Some(Immediate::Data(method_data)),
        Op::MethodCall.default_effects(),
        Some(span),
    );
    release_magic_set_value_after_call(ctx, value, span);
}

/// Releases an owning RHS temporary after the `__set` call has consumed it.
fn release_magic_set_value_after_call(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) {
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
}

/// Returns true when the runtime class of `object` declares a `__propset_<property>` set-hook
/// accessor, meaning a write to `property` should be routed through it.
fn set_hook_receiver_has_accessor(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
) -> bool {
    let PhpType::Object(class_name) = ctx.builder.value_php_type(object).codegen_repr() else {
        return false;
    };
    let normalized = class_name.trim_start_matches('\\');
    ctx.classes.get(normalized).is_some_and(|info| {
        info.methods
            .contains_key(&php_symbol_key(&property_hook_set_method(property)))
    })
}

/// Lowers a write to a set-hooked property as a call to its `__propset_<p>($value)` accessor,
/// passing the assigned value as the single argument and releasing it if it was an owning temporary.
fn lower_property_hook_set(
    ctx: &mut LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
    value: LoweredValue,
    span: Span,
) {
    let method_data = ctx.intern_string(&property_hook_set_method(property));
    ctx.emit_void(
        Op::MethodCall,
        vec![object, value.value],
        Some(Immediate::Data(method_data)),
        Op::MethodCall.default_effects(),
        Some(span),
    );
    release_magic_set_value_after_call(ctx, value, span);
}

/// Converts array literals to hash storage when a declared object property requires assoc storage.
fn contextualize_property_array_assignment(
    ctx: &mut LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
    lowered: LoweredValue,
    value_expr: &Expr,
    span: Span,
) -> LoweredValue {
    let php_type = ctx.builder.value_php_type(lowered.value);
    if !matches!(value_expr.kind, ExprKind::ArrayLiteral(_)) {
        return lowered;
    }
    if !matches!(php_type.codegen_repr(), PhpType::Array(_)) {
        return lowered;
    }
    let Some(contextual_ty) = object_property_type(ctx, object, property) else {
        return lowered;
    };
    let contextual_ty = contextual_ty.codegen_repr();
    if !matches!(contextual_ty, PhpType::AssocArray { .. }) {
        return lowered;
    }
    ctx.emit_value(
        Op::ArrayToHash,
        vec![lowered.value],
        None,
        contextual_ty,
        Op::ArrayToHash.default_effects(),
        Some(span),
    )
}

/// Lowers a static property write.
fn lower_static_property_assign(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    value: &Expr,
    span: Span,
) {
    let value = lower_expr(ctx, value);
    let value = coerce_mixed_static_property_store_value(ctx, receiver, property, value, span);
    store_static_property(ctx, receiver, property, value.value, span);
}

/// Unboxes a Mixed-typed value into a scalar static property's declared storage type
/// (e.g. a boxed foreach key stored through `foreach ($a as R::$k => $v)`), reusing the
/// standard scalar coercion helpers. Non-Mixed values and non-scalar declared types are
/// returned unchanged so every combination the backend already supports (tagged scalars,
/// Mixed/union slots, array init) keeps its existing lowering.
fn coerce_mixed_static_property_store_value(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    value: LoweredValue,
    span: Span,
) -> LoweredValue {
    if ctx.builder.value_php_type(value.value).codegen_repr() != PhpType::Mixed {
        return value;
    }
    let Some(property_ty) = static_property_type(ctx, receiver, property) else {
        return value;
    };
    match property_ty.codegen_repr() {
        PhpType::Int | PhpType::Bool => coerce_to_int(ctx, value, Some(span)),
        PhpType::Float => coerce_to_float(ctx, value, Some(span)),
        PhpType::Str => coerce_to_string(ctx, value, Some(span)),
        _ => value,
    }
}

/// Lowers `Class::$prop[] = value`.
fn lower_static_property_array_push(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    value: &Expr,
    span: Span,
) {
    if let Some(property_ty) =
        static_property_type(ctx, receiver, property).filter(is_indexed_array_type)
    {
        let property_value = load_static_property_as(ctx, receiver, property, property_ty, span);
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::ArrayPush,
            vec![property_value.value, value.value],
            None,
            Op::ArrayPush.default_effects(),
            Some(span),
        );
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }

    let property_value = load_static_property(ctx, receiver, property, span);
    let value = lower_expr(ctx, value);
    ctx.emit_void(
        Op::RuntimeCall,
        vec![property_value.value, value.value],
        None,
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Lowers `Class::$prop[index] = value`.
fn lower_static_property_array_assign(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
) {
    if let Some(property_ty) =
        static_property_type(ctx, receiver, property).filter(is_indexed_array_type)
    {
        let array_ty = property_ty.clone();
        let property_value = load_static_property_as(ctx, receiver, property, property_ty, span);
        let index = lower_expr(ctx, index);
        let value = lower_expr(ctx, value);
        let value = coerce_indexed_array_set_value(ctx, &array_ty, value, Some(span));
        ctx.emit_void(
            Op::ArraySet,
            vec![property_value.value, index.value, value.value],
            None,
            Op::ArraySet.default_effects(),
            Some(span),
        );
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }

    // A string/int/mixed-key write into an ASSOCIATIVE static array — including one de-packed to a
    // hash by a reference-alias (`self::$a[$dir] = &self::$a[$k]`). Mirror the local string-key
    // hash write: load as a hash, `HashSet` in place, store the (possibly relocated) hash back.
    // `HashSet` mutates operand 0 and codegen writes the relocation into its SSA home, so storing
    // that same loaded value back is a same-container round-trip (codegen suppresses the release).
    if let Some(property_ty) =
        static_property_type(ctx, receiver, property).filter(is_assoc_array_type)
    {
        let property_value = load_static_property_as(ctx, receiver, property, property_ty, span);
        let index = lower_expr(ctx, index);
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::HashSet,
            vec![property_value.value, index.value, value.value],
            None,
            Op::HashSet.default_effects(),
            Some(span),
        );
        release_persisted_string_operand(ctx, index, span);
        release_persisted_string_operand(ctx, value, span);
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }

    let property_value = if let Some(property_ty) =
        static_property_type(ctx, receiver, property)
            .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty))
    {
        load_static_property_as(ctx, receiver, property, property_ty, span)
    } else {
        load_static_property(ctx, receiver, property, span)
    };
    let index = lower_expr(ctx, index);
    let value = lower_expr(ctx, value);
    ctx.emit_void(
        Op::RuntimeCall,
        vec![property_value.value, index.value, value.value],
        None,
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Lowers a write through a dynamic-named static property (`self::${$n} = v`,
/// `self::${$n}[$k] = v`, `self::${$n}[] = v`).
///
/// The runtime property-name expression is evaluated exactly once and coerced to a string, then
/// reused for both the load and the store so its side effects run once and both compare-chains
/// select the same candidate. Array-element and push forms mirror the static-named indexed path:
/// the array is loaded, mutated with `ArraySet`/`ArrayPush`, and stored back to the selected
/// symbol (codegen suppresses the store's `release_previous` for this same-array write-back).
/// These forms are supported when the receiver class's static properties share one indexed-array
/// type; a heterogeneous class, an associative/`Mixed` array, or a string key remains a loud
/// codegen error (associative static-property storage is a pre-existing, shared limitation). The
/// direct form stores the value straight into the selected symbol.
fn lower_dynamic_static_property_write(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &Expr,
    index: Option<&Expr>,
    append: bool,
    value: &Expr,
    span: Span,
) {
    let name_value = lower_expr(ctx, property);
    let name_str = coerce_to_string(ctx, name_value, Some(span));
    let name = name_str.value;
    let common_ty = dynamic_static_property_common_type(ctx, receiver);

    if append {
        // A homogeneous indexed-array class pushes and stores the array back; any other common
        // type uses the generic runtime append (valid EIR; degrades/fatals at runtime instead of a
        // silent lost write).
        if let Some(array_ty) = common_ty.clone().filter(is_indexed_array_type) {
            let array_value = load_dynamic_static_property_as(ctx, receiver, name, array_ty, span);
            let value = lower_expr(ctx, value);
            ctx.emit_void(
                Op::ArrayPush,
                vec![array_value.value, value.value],
                None,
                Op::ArrayPush.default_effects(),
                Some(span),
            );
            store_dynamic_static_property(ctx, receiver, name, array_value.value, span);
            return;
        }
        let array_ty = common_ty
            .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty))
            .unwrap_or(PhpType::Mixed);
        let array_value = load_dynamic_static_property_as(ctx, receiver, name, array_ty, span);
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::RuntimeCall,
            vec![array_value.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        return;
    }

    if let Some(index) = index {
        lower_dynamic_static_array_element_set(
            ctx, receiver, name, common_ty, index, value, span,
        );
        return;
    }

    let value = lower_expr(ctx, value);
    store_dynamic_static_property(ctx, receiver, name, value.value, span);
}

/// Lowers the array-element write of a dynamic-named static property (`self::${$n}[$k] = v`).
///
/// Mirrors the static-named indexed `lower_static_property_array_assign`: a homogeneous
/// indexed-array class loads the array, mutates it in place with `ArraySet`, then stores it back
/// (codegen suppresses `release_previous` because the stored value is the same loaded array). Any
/// other common type uses the generic runtime array-access call (valid EIR). A string key into an
/// `Array`-typed static property remains a loud codegen error, exactly as for the static-named
/// path — associative storage in a typed `array` static property is a pre-existing, shared
/// limitation.
fn lower_dynamic_static_array_element_set(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    name: crate::ir::ValueId,
    common_ty: Option<PhpType>,
    index: &Expr,
    value: &Expr,
    span: Span,
) {
    if let Some(array_ty) = common_ty.clone().filter(is_indexed_array_type) {
        let element_array_ty = array_ty.clone();
        let array_value = load_dynamic_static_property_as(ctx, receiver, name, array_ty, span);
        let index_value = lower_expr(ctx, index);
        let value_value = lower_expr(ctx, value);
        let value_value =
            coerce_indexed_array_set_value(ctx, &element_array_ty, value_value, Some(span));
        ctx.emit_void(
            Op::ArraySet,
            vec![array_value.value, index_value.value, value_value.value],
            None,
            Op::ArraySet.default_effects(),
            Some(span),
        );
        store_dynamic_static_property(ctx, receiver, name, array_value.value, span);
        return;
    }

    let array_ty = common_ty
        .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty))
        .unwrap_or(PhpType::Mixed);
    let array_value = load_dynamic_static_property_as(ctx, receiver, name, array_ty, span);
    let index_value = lower_expr(ctx, index);
    let value_value = lower_expr(ctx, value);
    ctx.emit_void(
        Op::RuntimeCall,
        vec![array_value.value, index_value.value, value_value.value],
        None,
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Loads a dynamic-named static property (`self::${$n}`) using a pre-computed runtime name value.
///
/// `name` is a `Str` SSA value produced once by the caller. The receiver's concrete class name is
/// carried as the immediate so codegen can enumerate that class's declared static properties.
fn load_dynamic_static_property_as(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    name: crate::ir::ValueId,
    php_type: PhpType,
    span: Span,
) -> LoweredValue {
    let class_name =
        static_receiver_class_name(ctx, receiver).unwrap_or_else(|| receiver_name(receiver));
    let data = ctx.intern_string(&class_name);
    ctx.emit_value(
        Op::LoadDynamicStaticProperty,
        vec![name],
        Some(Immediate::Data(data)),
        php_type,
        Op::LoadDynamicStaticProperty.default_effects(),
        Some(span),
    )
}

/// Stores a value into a dynamic-named static property (`self::${$n} = v`) using a pre-computed
/// runtime name value. The receiver's concrete class name is the immediate; codegen dispatches on
/// the runtime name across the class's static properties and writes the matching symbol.
fn store_dynamic_static_property(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    name: crate::ir::ValueId,
    value: crate::ir::ValueId,
    span: Span,
) {
    let class_name =
        static_receiver_class_name(ctx, receiver).unwrap_or_else(|| receiver_name(receiver));
    let data = ctx.intern_string(&class_name);
    ctx.emit_void(
        Op::StoreDynamicStaticProperty,
        vec![name, value],
        Some(Immediate::Data(data)),
        Op::StoreDynamicStaticProperty.default_effects(),
        Some(span),
    );
}

/// Returns the common declared type of a receiver class's static properties, or `Mixed` when they
/// are heterogeneous. Used to pick the load/store shape for a dynamic-named static property write.
/// Returns `None` when the receiver class is not statically resolvable.
fn dynamic_static_property_common_type(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
) -> Option<PhpType> {
    let class_name = static_receiver_class_name(ctx, receiver)?;
    let class_info = ctx.classes.get(class_name.as_str())?;
    let mut types = class_info
        .static_properties
        .iter()
        .map(|(_, property_ty)| normalize_value_php_type(property_ty.codegen_repr()));
    let first = types.next()?;
    if types.all(|ty| ty == first) {
        Some(first)
    } else {
        Some(PhpType::Mixed)
    }
}

/// Lowers `$object->prop[] = value` (array append onto an object property).
///
/// Handles two property shapes directly: a concrete indexed `Array(elem)`
/// property (via `Op::ArrayPush`) and a concrete associative `AssocArray`
/// property (via a 2-operand `Op::RuntimeCall` that lands on the backend's
/// hash-append lowering, exactly like a local `$arr[] = $v` push). Nullable or
/// union `?array` properties (whose codegen type collapses to `Mixed`) are
/// intentionally deferred and left on the generic fallback so they remain a loud
/// compile error rather than a silent element drop.
fn lower_property_array_push(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    value: &Expr,
    span: Span,
) {
    let object = lower_expr(ctx, object);
    if let Some(property_ty) =
        object_property_type(ctx, object.value, property).filter(is_indexed_array_type)
    {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        let property_value =
            crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(span));
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::ArrayPush,
            vec![property_value.value, value.value],
            None,
            Op::ArrayPush.default_effects(),
            Some(span),
        );
        release_property_array_insert_value_after_retain(ctx, &property_ty, value, span);
        ctx.emit_void(
            Op::PropSet,
            vec![object.value, property_value.value],
            Some(Immediate::Data(data)),
            Op::PropSet.default_effects(),
            Some(span),
        );
        release_rewritten_property_value_after_retaining_store(ctx, &property_ty, property_value, span);
        return;
    }

    if let Some(property_ty) =
        object_property_type(ctx, object.value, property).filter(is_assoc_array_type)
    {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        let property_value =
            crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(span));
        let value = lower_expr(ctx, value);
        // Append with the next integer key. A 2-operand `RuntimeCall` on the hash-backed
        // property value lands on the backend's hash-append lowering (the
        // `(AssocArray, Void)` arm), exactly like a local associative-array push
        // `$arr[] = $v`; the backend inlines the next-int-key scan and calls
        // `__rt_hash_set`, which COW-splits and may relocate the table (its updated
        // pointer is written back into `property_value`'s slot, so the `PropSet` below
        // stores the correct pointer).
        ctx.emit_void(
            Op::RuntimeCall,
            vec![property_value.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        release_property_array_insert_value_after_retain(ctx, &property_ty, value, span);
        ctx.emit_void(
            Op::PropSet,
            vec![object.value, property_value.value],
            Some(Immediate::Data(data)),
            Op::PropSet.default_effects(),
            Some(span),
        );
        release_rewritten_property_value_after_retaining_store(ctx, &property_ty, property_value, span);
        return;
    }

    let value = lower_expr(ctx, value);
    let data = ctx.intern_string(property);
    ctx.emit_void(
        Op::RuntimeCall,
        vec![object.value, value.value],
        Some(Immediate::Data(data)),
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Lowers `$object->prop[index] = value`.
fn lower_property_array_assign(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
) {
    let object = lower_expr(ctx, object);
    if let Some(property_ty) =
        object_property_type(ctx, object.value, property).filter(is_indexed_array_type)
    {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        let property_value =
            crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(span));
        let index = lower_expr(ctx, index);
        let value = lower_expr(ctx, value);
        let value = coerce_indexed_array_set_value(ctx, &property_ty, value, Some(span));
        ctx.emit_void(
            Op::ArraySet,
            vec![property_value.value, index.value, value.value],
            None,
            Op::ArraySet.default_effects(),
            Some(span),
        );
        release_property_array_insert_value_after_retain(ctx, &property_ty, value, span);
        ctx.emit_void(
            Op::PropSet,
            vec![object.value, property_value.value],
            Some(Immediate::Data(data)),
            Op::PropSet.default_effects(),
            Some(span),
        );
        release_rewritten_property_value_after_retaining_store(ctx, &property_ty, property_value, span);
        return;
    }
    if let Some(property_ty) =
        object_property_type(ctx, object.value, property).filter(is_assoc_array_type)
    {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        let property_value =
            crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(span));
        let index = lower_expr(ctx, index);
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::HashSet,
            vec![property_value.value, index.value, value.value],
            None,
            Op::HashSet.default_effects(),
            Some(span),
        );
        release_property_array_insert_value_after_retain(ctx, &property_ty, value, span);
        ctx.emit_void(
            Op::PropSet,
            vec![object.value, property_value.value],
            Some(Immediate::Data(data)),
            Op::PropSet.default_effects(),
            Some(span),
        );
        release_rewritten_property_value_after_retaining_store(ctx, &property_ty, property_value, span);
        return;
    }

    // Gradual typing: `array|false`/`?array` property (PHP auto-vivifies false/null into an
    // array on first indexed write; checker-accepted in `stmt_check/assignments/properties.rs`).
    // `object_property_type()` collapses `Union` to its `Mixed` codegen representation, so the
    // raw declared type is checked here directly (before that collapse) to detect this case.
    // Union storage is already a boxed-cell pointer at runtime (same representation as `Mixed`),
    // so a plain `Op::PropGet` fetch (no re-store afterward — `__rt_mixed_array_set` mutates the
    // fetched cell's tag/payload in place) plus the shared boxed-Mixed writer is sufficient; no
    // new EIR op is needed.
    if let Some(raw_property_ty) = raw_object_property_type(ctx, object.value, property).filter(
        |ty| matches!(ty, PhpType::Union(members) if is_gradual_array_bool_void_union(members)),
    ) {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            raw_property_ty.codegen_repr(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        let index = lower_expr(ctx, index);
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::RuntimeCall,
            vec![property_value.value, index.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        return;
    }

    if let Some(property_ty) =
        object_property_type(ctx, object.value, property)
            .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty))
    {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty,
            Op::PropGet.default_effects(),
            Some(span),
        );
        let index = lower_expr(ctx, index);
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::RuntimeCall,
            vec![property_value.value, index.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        return;
    }

    let index = lower_expr(ctx, index);
    let value = lower_expr(ctx, value);
    let data = ctx.intern_string(property);
    ctx.emit_void(
        Op::RuntimeCall,
        vec![object.value, index.value, value.value],
        Some(Immediate::Data(data)),
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Releases a temporary assigned into an object property after `PropSet` retains or boxes it.
pub(crate) fn release_property_assignment_source_after_retaining_store(
    ctx: &mut LoweringContext<'_, '_>,
    property_ty: &PhpType,
    value: LoweredValue,
    span: Span,
) {
    if !ctx.value_is_owning_temporary(value) {
        return;
    }
    if !property_store_keeps_independent_ref(property_ty, &ctx.builder.value_php_type(value.value)) {
        return;
    }
    crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
}

/// Releases an element temporary after a property-array write retains it for storage.
fn release_property_array_insert_value_after_retain(
    ctx: &mut LoweringContext<'_, '_>,
    property_ty: &PhpType,
    value: LoweredValue,
    span: Span,
) {
    let Some(elem_ty) = indexed_property_array_element_type(property_ty) else {
        return;
    };
    if matches!(elem_ty.codegen_repr(), PhpType::Mixed | PhpType::Callable) {
        return;
    }
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
}

/// Releases the loaded property value after rewriting it through a retaining `PropSet`.
fn release_rewritten_property_value_after_retaining_store(
    ctx: &mut LoweringContext<'_, '_>,
    property_ty: &PhpType,
    property_value: LoweredValue,
    span: Span,
) {
    if property_ty.codegen_repr().is_refcounted() {
        crate::ir_lower::ownership::release_if_owned(ctx, property_value, Some(span));
    }
}

/// Returns whether a property store creates a distinct retained/boxed owner for the value.
fn property_store_keeps_independent_ref(property_ty: &PhpType, value_ty: &PhpType) -> bool {
    let property_ty = property_ty.codegen_repr();
    let value_ty = value_ty.codegen_repr();
    if matches!((&property_ty, &value_ty), (PhpType::Mixed, PhpType::Mixed)) {
        return false;
    }
    if matches!(property_ty, PhpType::Str) {
        return true;
    }
    property_ty.is_refcounted()
}

/// Returns the element type for property arrays that use retaining indexed/hash helpers.
fn indexed_property_array_element_type(property_ty: &PhpType) -> Option<PhpType> {
    match property_ty.codegen_repr() {
        PhpType::Array(elem_ty) => Some(elem_ty.codegen_repr()),
        PhpType::AssocArray { value, .. } => Some(value.codegen_repr()),
        _ => None,
    }
}

/// Emits a no-op marker for declaration-only or frontend-only statements.
fn lower_noop(ctx: &mut LoweringContext<'_, '_>, span: Span) {
    ctx.emit_void(Op::Nop, Vec::new(), None, Op::Nop.default_effects(), Some(span));
}

/// Records a function variant group in high-level EIR metadata form.
fn lower_function_variant_group(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    variants: &[String],
    span: Span,
) {
    let label = format!("{}:{}", name, variants.join(","));
    let data = ctx.intern_string(&label);
    ctx.emit_void(
        Op::FunctionVariantDispatch,
        Vec::new(),
        Some(Immediate::Data(data)),
        Op::FunctionVariantDispatch.default_effects(),
        Some(span),
    );
}

/// Records one selected function variant.
fn lower_function_variant_mark(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    variant: &str,
    span: Span,
) {
    let label = format!("{}:{}", name, variant);
    let data = ctx.intern_string(&label);
    ctx.emit_void(
        Op::FunctionVariantMark,
        Vec::new(),
        Some(Immediate::Data(data)),
        Op::FunctionVariantMark.default_effects(),
        Some(span),
    );
}

/// Emits a branch to `target` if the current block can still fall through.
fn branch_to(ctx: &mut LoweringContext<'_, '_>, target: BlockId) {
    if !ctx.builder.insertion_block_is_terminated() {
        ctx.builder.terminate(Terminator::Br { target, args: Vec::new() });
    }
}

/// Finds the active loop target for a one-based break/continue level.
fn loop_target(ctx: &LoweringContext<'_, '_>, level: usize) -> Option<LoopFrame> {
    let level = level.max(1);
    ctx.loop_stack
        .len()
        .checked_sub(level)
        .and_then(|index| ctx.loop_stack.get(index).copied())
}

/// Selects the strongest array write opcode valid for a lowered array value.
fn array_set_op(ir_type: IrType) -> Op {
    match ir_type {
        IrType::Heap(crate::ir::IrHeapKind::Array) => Op::ArraySet,
        IrType::Heap(crate::ir::IrHeapKind::Hash) => Op::HashSet,
        IrType::Heap(crate::ir::IrHeapKind::Buffer) => Op::BufferSet,
        _ => Op::RuntimeCall,
    }
}

/// Returns true when a lowered index value is a boxed `Mixed`/`Union` cell that
/// may hold either an integer or a string array key (e.g. a foreach loop key,
/// which `Op::IterCurrentKey` always produces as Mixed). Such writes go through
/// `Op::ArraySetMixedKey` so the key tag is dispatched at runtime instead of
/// coercing it to int (which would collapse a string key onto int 0).
fn index_is_boxed_mixed_key(ir_type: IrType) -> bool {
    matches!(
        ir_type,
        IrType::Heap(crate::ir::IrHeapKind::Mixed)
            | IrType::Heap(crate::ir::IrHeapKind::Union)
    )
}

/// Returns true when the index expression is a foreach loop key known to hold an
/// integer at runtime (its source was a concretely-indexed array), so the
/// destination write can keep the indexed `ArraySet` path with int coercion
/// instead of promoting to a hash. See `LoweringContext::mark_foreach_int_key`.
fn index_is_foreach_int_key(ctx: &LoweringContext<'_, '_>, index: &Expr) -> bool {
    if let ExprKind::Variable(name) = &index.kind {
        return ctx.is_foreach_int_key(name);
    }
    false
}

/// Extracts an integer switch case value from literal cases.
fn int_case_value(expr: &Expr) -> Option<i64> {
    match &expr.kind {
        ExprKind::IntLiteral(value) => Some(*value),
        ExprKind::BoolLiteral(value) => Some(i64::from(*value)),
        _ => None,
    }
}

/// Emits a boolean constant value.
fn emit_const_bool(
    ctx: &mut LoweringContext<'_, '_>,
    value: bool,
    span: Option<Span>,
) -> LoweredValue {
    let value = ctx
        .builder
        .emit_with_effects(
            Op::ConstBool,
            Vec::new(),
            Some(Immediate::Bool(value)),
            IrType::I64,
            PhpType::Bool,
            Ownership::NonHeap,
            Op::ConstBool.default_effects(),
            span,
        )
        .expect("const_bool produces a value");
    LoweredValue { value, ir_type: IrType::I64 }
}

/// Emits a null sentinel value.
fn emit_null_value(ctx: &mut LoweringContext<'_, '_>, span: Option<Span>) -> LoweredValue {
    let value = ctx
        .builder
        .emit_with_effects(
            Op::ConstNull,
            Vec::new(),
            None,
            IrType::I64,
            PhpType::Void,
            Ownership::NonHeap,
            Op::ConstNull.default_effects(),
            span,
        )
        .expect("const_null produces a value");
    LoweredValue { value, ir_type: IrType::I64 }
}

/// Coerces a value to the current function return storage type when needed.
fn coerce_to_return_type(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    if let Some(value) = coerce_container_to_return_type(ctx, value, span) {
        return value;
    }
    if value.ir_type == ctx.return_type {
        return value;
    }
    match ctx.return_type {
        // The numeric unboxes below produce detached scalars, so an owned refcounted
        // source (e.g. a `hash_get` Mixed element box returned under a declared `: int`)
        // must be released once the coercion consumed it — otherwise every call leaks
        // the box (`coerce_to_string`'s Mixed arm already follows this convention).
        IrType::I64 => {
            let result = coerce_to_int(ctx, value, span);
            release_coercion_source_if_owned(ctx, value, result, span);
            result
        }
        IrType::F64 => {
            let result = coerce_to_float(ctx, value, span);
            release_coercion_source_if_owned(ctx, value, result, span);
            result
        }
        IrType::Str => coerce_to_string(ctx, value, span),
        IrType::TaggedScalar => coerce_to_tagged_scalar(ctx, value, span),
        IrType::Heap(_) if ctx.return_php_type.codegen_repr() == PhpType::Mixed => {
            let boxed = ctx.emit_value(
                Op::MixedBox,
                vec![value.value],
                None,
                ctx.return_php_type.clone(),
                Op::MixedBox.default_effects(),
                span,
            );
            release_coercion_source_if_owned(ctx, value, boxed, span);
            boxed
        }
        IrType::Heap(_) => {
            // Unboxing a Mixed cell into a concrete heap return type rebuilds/clones
            // a freshly owned payload, so it never aliases the source box.
            let unboxed = ctx.emit_value(
                Op::RuntimeCall,
                vec![value.value],
                None,
                ctx.return_php_type.clone(),
                effects_lookup::runtime_effects(),
                span,
            );
            release_coercion_source_if_owned(ctx, value, unboxed, span);
            unboxed
        }
        IrType::Void => value,
    }
}

/// Releases an owned source temporary consumed by a return-type coercion.
///
/// The heap return coercions (`MixedBox` for a Mixed return, the unbox
/// `RuntimeCall` for a concrete heap return) allocate a fresh owned result that
/// retains or clones the payload independently of the source. When the source is
/// an owning temporary (for example a Mixed box taken from a ternary merge temp),
/// it must be released once the coercion has produced its own reference, otherwise
/// it leaks on every return. The release is skipped when the coercion returned the
/// source unchanged so no double-release can occur.
fn release_coercion_source_if_owned(
    ctx: &mut LoweringContext<'_, '_>,
    source: LoweredValue,
    result: LoweredValue,
    span: Option<Span>,
) {
    if result.value != source.value && ctx.value_needs_release_after_retaining_store(source) {
        crate::ir_lower::ownership::release_if_owned(ctx, source, span);
    }
}

/// Coerces an integer-or-null value into the two-word tagged-scalar return shape.
fn coerce_to_tagged_scalar(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    if value.ir_type == IrType::TaggedScalar {
        return value;
    }
    if matches!(ctx.builder.value_php_type(value.value).codegen_repr(), PhpType::Void) {
        return ctx.emit_value(
            Op::ConstNull,
            Vec::new(),
            None,
            PhpType::TaggedScalar,
            Op::ConstNull.default_effects(),
            span,
        );
    }
    ctx.emit_value(
        Op::RuntimeCall,
        vec![value.value],
        None,
        PhpType::TaggedScalar,
        effects_lookup::runtime_effects(),
        span,
    )
}

/// Widens returned container payload storage to the current function return contract.
fn coerce_container_to_return_type(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> Option<LoweredValue> {
    let source_ty = ctx.builder.value_php_type(value.value).codegen_repr();
    let return_ty = ctx.return_php_type.codegen_repr();
    let op = match (source_ty, return_ty.clone()) {
        (PhpType::Array(source_elem), PhpType::Array(return_elem))
            if source_elem.codegen_repr() != PhpType::Mixed
                && return_elem.codegen_repr() == PhpType::Mixed =>
        {
            Op::ArrayToMixed
        }
        (
            PhpType::AssocArray { value: source_value, .. },
            PhpType::AssocArray { value: return_value, .. },
        ) if source_value.codegen_repr() != PhpType::Mixed
            && return_value.codegen_repr() == PhpType::Mixed =>
        {
            Op::HashToMixed
        }
        // A hash returned under a declared `array` contract typed `Array(Mixed)` (e.g. a
        // by-reference literal desugar on one return path joined with a differently-shaped
        // return elsewhere) keeps its hash storage: `HashToMixed` widens the bucket values
        // to boxed Mixed in place and the result is re-stamped with the return contract.
        // Every Array-typed consumer is heap-kind-aware (kind-probing Mixed boxing,
        // `__rt_decref_any` releases, `__rt_array_free_deep`'s kind-3 delegation), so the
        // static Array view over runtime hash storage reads, counts, and frees correctly.
        (PhpType::AssocArray { .. }, PhpType::Array(return_elem))
            if return_elem.codegen_repr() == PhpType::Mixed =>
        {
            Op::HashToMixed
        }
        _ => return None,
    };
    Some(ctx.emit_value(
        op,
        vec![value.value],
        None,
        return_ty,
        op.default_effects(),
        span,
    ))
}

/// Coerces a value to integer storage.
fn coerce_to_int(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    match value.ir_type {
        IrType::I64 => value,
        IrType::F64 => ctx.emit_value(
            Op::FToI,
            vec![value.value],
            None,
            PhpType::Int,
            Op::FToI.default_effects(),
            span,
        ),
        IrType::Str => ctx.emit_value(
            Op::StrToI,
            vec![value.value],
            None,
            PhpType::Int,
            Op::StrToI.default_effects(),
            span,
        ),
        _ => ctx.emit_value(
            Op::Cast,
            vec![value.value],
            Some(Immediate::CastTarget(IrType::I64)),
            PhpType::Int,
            Op::Cast.default_effects(),
            span,
        ),
    }
}

/// Coerces a value to float storage.
fn coerce_to_float(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    match value.ir_type {
        IrType::F64 => value,
        IrType::I64 => ctx.emit_value(
            Op::IToF,
            vec![value.value],
            None,
            PhpType::Float,
            Op::IToF.default_effects(),
            span,
        ),
        IrType::Str => ctx.emit_value(
            Op::StrToF,
            vec![value.value],
            None,
            PhpType::Float,
            Op::StrToF.default_effects(),
            span,
        ),
        _ => ctx.emit_value(
            Op::Cast,
            vec![value.value],
            Some(Immediate::CastTarget(IrType::F64)),
            PhpType::Float,
            Op::Cast.default_effects(),
            span,
        ),
    }
}

/// Coerces a value to string storage.
fn coerce_to_string(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    match value.ir_type {
        IrType::Str => value,
        IrType::I64 | IrType::TaggedScalar => ctx.emit_value(
            Op::IToStr,
            vec![value.value],
            None,
            PhpType::Str,
            Op::IToStr.default_effects(),
            span,
        ),
        IrType::F64 => ctx.emit_value(
            Op::FToStr,
            vec![value.value],
            None,
            PhpType::Str,
            Op::FToStr.default_effects(),
            span,
        ),
        _ => {
            let result = ctx.emit_value(
                Op::Cast,
                vec![value.value],
                Some(Immediate::CastTarget(IrType::Str)),
                PhpType::Str,
                Op::Cast.default_effects(),
                span,
            );
            // The Mixed/heap → string cast allocates a fresh, detached string copy
            // (`__rt_mixed_cast_string` persists the payload), so it never aliases
            // the source storage. An owned source temporary (for example a Mixed box
            // produced by a ternary merge temp) must be released here, otherwise it
            // leaks when the return value is coerced to a narrower string type.
            release_coercion_source_if_owned(ctx, value, result, span);
            result
        }
    }
}

/// Loads a static property value through a high-level EIR read.
fn load_static_property(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    span: Span,
) -> LoweredValue {
    load_static_property_as(ctx, receiver, property, PhpType::Mixed, span)
}

/// Loads a static property value using known PHP metadata.
fn load_static_property_as(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    php_type: PhpType,
    span: Span,
) -> LoweredValue {
    let name = format!("{}::{}", receiver_name(receiver), property);
    let data = ctx.intern_string(&name);
    ctx.emit_value(
        Op::LoadStaticProperty,
        Vec::new(),
        Some(Immediate::Data(data)),
        php_type,
        Op::LoadStaticProperty.default_effects(),
        Some(span),
    )
}

/// Stores a static property value through a high-level EIR write.
fn store_static_property(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    value: crate::ir::ValueId,
    span: Span,
) {
    let name = format!("{}::{}", receiver_name(receiver), property);
    let data = ctx.intern_string(&name);
    ctx.emit_void(
        Op::StoreStaticProperty,
        vec![value],
        Some(Immediate::Data(data)),
        Op::StoreStaticProperty.default_effects(),
        Some(span),
    );
}

/// Formats a static receiver for metadata immediates.
fn receiver_name(receiver: &StaticReceiver) -> String {
    match receiver {
        StaticReceiver::Named(name) => name.as_str().to_string(),
        StaticReceiver::Self_ => "self".to_string(),
        StaticReceiver::Static => "static".to_string(),
        StaticReceiver::Parent => "parent".to_string(),
    }
}

/// Resolves the declared PHP type of a static property for statement lowering.
fn static_property_type(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
) -> Option<PhpType> {
    let class_name = static_receiver_class_name(ctx, receiver)?;
    ctx.classes
        .get(class_name.as_str())?
        .static_properties
        .iter()
        .find(|(name, _)| name == property)
        .map(|(_, property_ty)| normalize_value_php_type(property_ty.codegen_repr()))
}

/// Resolves a static receiver to a concrete class name when lexical metadata is available.
fn static_receiver_class_name(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
) -> Option<String> {
    match receiver {
        StaticReceiver::Named(name) => Some(name.as_str().trim_start_matches('\\').to_string()),
        StaticReceiver::Self_ | StaticReceiver::Static => ctx.current_class.clone(),
        StaticReceiver::Parent => {
            let current = ctx.current_class.as_deref()?;
            ctx.classes.get(current).and_then(|class_info| class_info.parent.clone())
        }
    }
}

/// Resolves the declared PHP type of an object property for statement lowering.
pub(crate) fn object_property_type(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
) -> Option<PhpType> {
    let object_ty = ctx.builder.value_php_type(object);
    let PhpType::Object(class_name) = object_ty else {
        return None;
    };
    ctx.classes
        .get(class_name.trim_start_matches('\\'))?
        .properties
        .iter()
        .find(|(name, _)| name == property)
        .map(|(_, property_ty)| normalize_value_php_type(property_ty.codegen_repr()))
}

/// Resolves the RAW declared PHP type of an object property, without the `Union` → `Mixed`
/// codegen-representation collapse `object_property_type()` applies. Needed to distinguish a
/// gradually-acceptable `array|false`/`?array` property (storage-wise `Mixed`-shaped, but only
/// checker-accepted for specific non-array members) from a genuinely `Mixed`-typed property.
fn raw_object_property_type(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
) -> Option<PhpType> {
    let object_ty = ctx.builder.value_php_type(object);
    let PhpType::Object(class_name) = object_ty else {
        return None;
    };
    ctx.classes
        .get(class_name.trim_start_matches('\\'))?
        .properties
        .iter()
        .find(|(name, _)| name == property)
        .map(|(_, property_ty)| property_ty.clone())
}

/// Returns true when a property type uses concrete indexed-array storage.
fn is_indexed_array_type(php_type: &PhpType) -> bool {
    matches!(php_type.codegen_repr(), PhpType::Array(_))
}

/// Returns true when a property type uses concrete associative-array storage.
fn is_assoc_array_type(php_type: &PhpType) -> bool {
    matches!(php_type.codegen_repr(), PhpType::AssocArray { .. })
}

/// Returns true when `members` is a union of array-family types (`Array`/`AssocArray`) plus
/// only `Bool`/`Void` (null) non-array alternatives, with at least one array-family member.
/// Mirrors `array_family_bool_void_union_accepts_write` in
/// `types::checker::stmt_check::assignments::properties` — the exact PHP auto-vivify matrix
/// `__rt_mixed_array_set` implements (false/null payloads vivify; other scalars keep the
/// pre-existing silent drop). Kept in lockstep with the checker acceptance test so a
/// checker-accepted write never reaches an unsupported EIR lowering path.
fn is_gradual_array_bool_void_union(members: &[PhpType]) -> bool {
    let mut saw_array = false;
    for member in members {
        match member {
            PhpType::Array(_) | PhpType::AssocArray { .. } => saw_array = true,
            PhpType::Bool | PhpType::Void => {}
            _ => return false,
        }
    }
    saw_array
}

/// Normalizes non-materializable statement metadata to the EIR null sentinel type.
fn normalize_value_php_type(php_type: PhpType) -> PhpType {
    if matches!(php_type, PhpType::Never) {
        PhpType::Void
    } else {
        php_type
    }
}
