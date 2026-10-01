//! Purpose:
//! Lowers PHP global constant references and static constant-introspection calls
//! encountered while converting AST expressions into EIR.
//!
//! Called from:
//! - `crate::ir_lower::expr::lower_expr()` and the function-call lowering path.
//!
//! Key details:
//! - `define("NAME", value)` updates the per-function lowering context in source
//!   order so later `ConstRef` expressions can keep precise PHP metadata.

use crate::ir::{Immediate, Op, Ownership};
use crate::ir_lower::context::{value_ir_type, LoweredValue, LoweringContext};
use crate::names::{php_symbol_key, Name};
use crate::parser::ast::{Expr, ExprKind};
use crate::types::PhpType;

/// Records `define("NAME", value)` constants for later source-order references.
pub(super) fn register_static_define_call(
    ctx: &mut LoweringContext<'_, '_>,
    name: &Name,
    args: &[Expr],
) {
    if php_symbol_key(name.as_str().trim_start_matches('\\')) != "define" || args.len() != 2 {
        return;
    }
    let ExprKind::StringLiteral(constant_name) = &args[0].kind else {
        return;
    };
    ctx.register_constant(
        constant_name.clone(),
        args[1].kind.clone(),
        constant_expr_type(&args[1].kind),
    );
}

/// Lowers `defined("NAME")` to a compile-time boolean when the name is literal.
pub(super) fn lower_static_defined_call(
    ctx: &mut LoweringContext<'_, '_>,
    name: &Name,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if php_symbol_key(name.as_str().trim_start_matches('\\')) != "defined" || args.len() != 1 {
        return None;
    }
    let ExprKind::StringLiteral(constant_name) = &args[0].kind else {
        return None;
    };
    let exists = ctx.constant_value(constant_name).is_some();
    if !exists && (ctx.has_eval_barrier() || ctx.eval_executed()) {
        // Barrier-free AOT evals can still define constants dynamically; the
        // probe needs the eval context, so make sure its slot exists.
        ctx.declare_eval_context_local();
        let data = ctx.intern_global_name(constant_name);
        return Some(ctx.emit_value(
            Op::EvalConstantExists,
            Vec::new(),
            Some(Immediate::Data(data)),
            PhpType::Bool,
            Op::EvalConstantExists.default_effects(),
            Some(expr.span),
        ));
    }
    let base = ctx.constants.base();
    let bare = constant_name.trim_start_matches('\\');
    if !base.contains_key(constant_name.as_str())
        && !base.contains_key(bare)
        && crate::ir_lower::context::is_runtime_define_name(bare)
    {
        // Only a builtin or an unconditional top-level declaration exists before its code runs.
        // Anything else is a `define()` somewhere -- in a branch, a function, an included file
        // -- and whether it has run is a runtime fact: folding it answered `false` on the second
        // pass through `if (!defined('X')) define('X', ...)`, which then warned and redefined.
        // The runtime `defined()` reads the define's own flag (`lower_defined`).
        return None;
    }
    Some(emit_typed_constant(
        ctx,
        Op::ConstBool,
        Some(Immediate::Bool(exists)),
        PhpType::Bool,
        expr,
    ))
}

/// Lowers `constant("NAME")` to exactly the EIR a bare `NAME` reference produces.
///
/// PHP's `constant()` performs a GLOBAL constant lookup: the name is already fully qualified,
/// so no namespace/`use` resolution applies and a leading `\` is stripped. Reusing
/// [`lower_const_ref`] means the call inherits the constant's real type and value — including
/// the prescanned `define()` metadata recorded earlier in source order — instead of an opaque
/// `mixed`. A `Foo::BAR` name keeps the runtime call, which reads the class-constant table; a
/// name nothing defines takes `lower_const_ref`'s throwing runtime lookup.
pub(super) fn lower_static_constant_call(
    ctx: &mut LoweringContext<'_, '_>,
    name: &Name,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if php_symbol_key(name.as_str().trim_start_matches('\\')) != "constant" || args.len() != 1 {
        return None;
    }
    let constant_name = static_constant_name_arg(&args[0])?;
    if constant_name.contains("::") {
        // A class constant or enum case: the runtime lookup reads the class-constant table.
        return None;
    }
    let referenced = Name::unqualified(constant_name.trim_start_matches('\\'));
    Some(lower_const_ref(ctx, &referenced, expr))
}

/// Returns the literal constant name from `constant()`'s single argument.
///
/// Accepts the positional form and the `name:` named-argument form; every other shape (a
/// runtime string, a spread) yields `None` so the caller falls back to the registry path,
/// which the checker has already rejected.
fn static_constant_name_arg(arg: &Expr) -> Option<&str> {
    match &arg.kind {
        ExprKind::StringLiteral(value) => Some(value.as_str()),
        ExprKind::NamedArg { name, value } if name == "name" => match &value.kind {
            ExprKind::StringLiteral(value) => Some(value.as_str()),
            _ => None,
        },
        _ => None,
    }
}

/// Lowers a constant reference through prescanned metadata or global storage fallback.
pub(super) fn lower_const_ref(
    ctx: &mut LoweringContext<'_, '_>,
    name: &Name,
    expr: &Expr,
) -> LoweredValue {
    if let Some((value, php_type)) = ctx.constant_value(name.as_str()) {
        return lower_constant_value(ctx, value, php_type, expr);
    }
    if let Some(php_type) = crate::types::predefined_constants::target_restricted_type(name.as_str())
    {
        return lower_target_undefined_constant(ctx, name, php_type, expr);
    }
    if !ctx.has_eval_barrier()
        && !ctx.eval_executed()
        && crate::types::predefined_constants::is_undefined_constant_read(name.as_str())
        && !crate::ir_lower::context::is_runtime_define_name(name.as_str().trim_start_matches('\\'))
    {
        // Nothing defines the name: php's read throws `Error: Undefined constant "NAME"`, which
        // the runtime lookup reproduces (and answers, should a dynamic `define()` add it).
        return lower_target_undefined_constant(ctx, name, PhpType::Mixed, expr);
    }
    if ctx.has_eval_barrier() || ctx.eval_executed() {
        // Barrier-free AOT evals can still define constants dynamically; the
        // fetch needs the eval context, so make sure its slot exists.
        ctx.declare_eval_context_local();
        let data = ctx.intern_global_name(name.as_str());
        return ctx.emit_value(
            Op::EvalConstantFetch,
            Vec::new(),
            Some(Immediate::Data(data)),
            PhpType::Mixed,
            Op::EvalConstantFetch.default_effects(),
            Some(expr.span),
        );
    }
    let data = ctx.intern_global_name(name.as_str());
    ctx.emit_value(
        Op::LoadGlobal,
        Vec::new(),
        Some(Immediate::GlobalName(data)),
        super::fallback_expr_type(expr),
        Op::LoadGlobal.default_effects(),
        Some(expr.span),
    )
}

/// Lowers a read of a constant php does not define: a catalogued one restricted to other targets
/// (`PHP_WINDOWS_VERSION_BUILD` off Windows), which prescan gave no value, or a name nothing
/// defines (typed `mixed`). The read goes to the runtime `constant()` lookup, which misses and
/// throws php's `Error: Undefined constant "NAME"`.
///
/// The checker still types the name from the catalog, so a guarded platform branch compiles;
/// the result is coerced to that type for the code that follows, which the throw never reaches.
fn lower_target_undefined_constant(
    ctx: &mut LoweringContext<'_, '_>,
    name: &Name,
    php_type: PhpType,
    expr: &Expr,
) -> LoweredValue {
    let bare = name.as_str().trim_start_matches('\\');
    let data = ctx.intern_string(bare);
    let name_value = emit_typed_constant(
        ctx,
        Op::ConstStr,
        Some(Immediate::Data(data)),
        PhpType::Str,
        expr,
    );
    let lookup = crate::ir::RuntimeFnId::Constant;
    let value = ctx.emit_value(
        Op::RuntimeCall,
        vec![name_value.value],
        Some(Immediate::RuntimeCall(crate::ir::RuntimeCallTarget::Function(lookup))),
        PhpType::Mixed,
        lookup.effects(),
        Some(expr.span),
    );
    match php_type {
        PhpType::Int => super::coerce_to_int(ctx, value, expr),
        PhpType::Float => super::coerce_to_float(ctx, value, expr),
        PhpType::Str => super::coerce_to_string(ctx, value, expr),
        _ => value,
    }
}

/// Lowers a prescanned constant value using its checker-visible PHP type.
fn lower_constant_value(
    ctx: &mut LoweringContext<'_, '_>,
    value: ExprKind,
    php_type: PhpType,
    expr: &Expr,
) -> LoweredValue {
    match value {
        ExprKind::IntLiteral(value) => emit_typed_constant(
            ctx,
            Op::ConstI64,
            Some(Immediate::I64(value)),
            php_type,
            expr,
        ),
        ExprKind::FloatLiteral(value) => emit_typed_constant(
            ctx,
            Op::ConstF64,
            Some(Immediate::F64(value)),
            php_type,
            expr,
        ),
        ExprKind::StringLiteral(value) => {
            let data = ctx.intern_string(&value);
            emit_typed_constant(ctx, Op::ConstStr, Some(Immediate::Data(data)), php_type, expr)
        }
        ExprKind::BoolLiteral(value) => emit_typed_constant(
            ctx,
            Op::ConstBool,
            Some(Immediate::Bool(value)),
            php_type,
            expr,
        ),
        ExprKind::Null => emit_typed_constant(ctx, Op::ConstNull, None, php_type, expr),
        other => {
            let synthetic = Expr::new(other, expr.span);
            super::lower_expr(ctx, &synthetic)
        }
    }
}

/// Emits a literal constant opcode with caller-supplied PHP metadata.
fn emit_typed_constant(
    ctx: &mut LoweringContext<'_, '_>,
    op: Op,
    immediate: Option<Immediate>,
    php_type: PhpType,
    expr: &Expr,
) -> LoweredValue {
    let ir_type = value_ir_type(&php_type);
    let value = ctx
        .builder
        .emit_with_effects(
            op,
            Vec::new(),
            immediate,
            ir_type,
            php_type.clone(),
            Ownership::for_php_type(&php_type),
            op.default_effects(),
            Some(expr.span),
        )
        .expect("constant opcode produces a value");
    LoweredValue { value, ir_type }
}

/// Returns the PHP type used for a compile-time constant expression.
fn constant_expr_type(kind: &ExprKind) -> PhpType {
    match kind {
        ExprKind::IntLiteral(_) => PhpType::Int,
        ExprKind::FloatLiteral(_) => PhpType::Float,
        ExprKind::StringLiteral(_) => PhpType::Str,
        ExprKind::BoolLiteral(false) => PhpType::False,
        ExprKind::BoolLiteral(true) => PhpType::Bool,
        ExprKind::Null => PhpType::Void,
        _ => PhpType::Int,
    }
}
