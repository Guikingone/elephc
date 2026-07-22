//! Purpose:
//! Shared type-check hooks for the callables-area class-reflection builtin homes.
//! Provides the common validation logic used by multiple homes to avoid duplication.
//!
//! Called from:
//! - `crate::builtins::callables::*` homes that set `check:` to one of these functions.
//!
//! Key details:
//! - Each hook receives a pre-populated `BuiltinCheckCtx`; for non-lazy homes args are
//!   already inferred by the registry common path before the hook runs.
//! - `check_class_like_exists` inspects `.kind` only (no infer) — the common path already
//!   inferred every arg before this hook is called.
//! - `check_class_relation` homes use `lazy_check: true`, so the hook performs its own
//!   inference in source order (matching the legacy arm).
//! - `check_declared_names` takes no args and returns `Array<Str>` unconditionally.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

/// Validates `class_exists` / `interface_exists` / `trait_exists` / `enum_exists` arguments.
///
/// Requires that the first argument is a string literal and, if present, the second argument
/// is a literal bool or int (the autoload flag). Returns `Bool` on success.
/// Arguments are pre-inferred by the registry common path before this hook runs.
pub(crate) fn check_class_like_exists(_cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    // A string literal (or `Name::class`) folds to a compile-time boolean using the
    // closed world before codegen; a NON-literal name is accepted too and lowered to
    // the closed-world registry probe (`lower_dynamic_class_like_exists` →
    // `__rt_class_exists`/`__rt_interface_exists`/`__rt_trait_exists`). elephc's
    // autoload is a compile-time pass, so the `$autoload` flag has no runtime effect
    // and accepts any value. The registry common path has already inferred the args.
    Ok(PhpType::Bool)
}

/// Validates `class_implements` / `class_parents` / `class_uses` arguments.
///
/// Infers the first argument and requires it to be an object or string literal.
/// If present, infers and validates the second argument (autoload flag) as a literal bool or int.
/// Returns the union `array<string,string>|bool` used by the PHP class-relation builtins.
/// This hook is called with `lazy_check: true` so inference happens here, not in the common path.
pub(crate) fn check_class_relation(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let first_ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    // An object or string literal resolves against the closed world at compile time. A
    // non-literal string or Mixed name (e.g. a `string|int` return threaded through a variable)
    // resolves at runtime through `__rt_class_relation_lookup`; the EIR feature scan sets
    // class_relation_introspection for any non-const argument, so the helper is always emitted.
    // Anything else (int, array, ...) cannot name a class and stays a compile-time error.
    let runtime_name_target = matches!(first_ty.codegen_repr(), PhpType::Mixed | PhpType::Str);
    if !matches!(first_ty, PhpType::Object(_))
        && !matches!(cx.args[0].kind, ExprKind::StringLiteral(_))
        && !runtime_name_target
    {
        return Err(CompileError::new(
            cx.span,
            &format!("{}() first argument must be an object or string literal in AOT mode", cx.name),
        ));
    }
    if let Some(autoload_arg) = cx.args.get(1) {
        cx.checker.infer_type(autoload_arg, cx.env)?;
        if !matches!(
            autoload_arg.kind,
            ExprKind::BoolLiteral(_) | ExprKind::IntLiteral(_)
        ) {
            return Err(CompileError::new(
                cx.span,
                &format!("{}() autoload argument must be a literal bool or int in AOT mode", cx.name),
            ));
        }
    }
    Ok(PhpType::Union(vec![
        PhpType::AssocArray {
            key: Box::new(PhpType::Str),
            value: Box::new(PhpType::Str),
        },
        PhpType::Bool,
    ]))
}

/// Returns `Array<Str>` for the zero-argument declared-names builtins.
///
/// The hook ignores its context because these builtins take no arguments; the registry
/// common path enforces arity = 0 before this hook runs.
pub(crate) fn check_declared_names(_cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}
