//! Purpose:
//! Home of the PHP `array_any` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The PHP golden signature is `fixed(&["array","callback"])` (exactly 2 required params).
//!   The legacy CHECK arm also required exactly 2 arguments; no arity override is needed.
//! - `check` accepts any array (keyed, or a gradual value holding one) and validates the
//!   predicate with the element and key types php passes it. Returns `PhpType::Bool`.
//! - Positional calls are lowered through `crate::array_find_prelude`, which walks any array and
//!   passes the key; the native runtime walks only a dense list.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "array_any",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayAny,
    ),
}

/// Validates the predicate callback for an `array_any` call and returns `PhpType::Bool`.
///
/// The first argument must be an array or a gradual value that may hold one. The callback is
/// validated with the element and key types as context. Arity (exactly 2 args) is
/// pre-validated by `check_arity`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let arr_ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    // Any array php accepts: a keyed one, or a gradual value that holds one (php raises the
    // TypeError at run time, as the prelude helper's `array` parameter does).
    if !matches!(
        arr_ty,
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Mixed | PhpType::Union(_)
    ) {
        return Err(CompileError::new(
            cx.span,
            &format!("{}() first argument must be array", cx.name),
        ));
    }
    // php calls the predicate with the value and then the key; a union names no element type.
    let element_source = if matches!(arr_ty, PhpType::Union(_)) {
        PhpType::Mixed
    } else {
        arr_ty.clone()
    };
    // A literal closure is checked against both (its surplus parameters are what it declares);
    // a named callback against the value alone, because a user function ignores the surplus key
    // the way php lets it, and the checker would otherwise refuse `array_any($a, 'is_big')`.
    let mut callback_arg_types = vec![crate::types::checker::builtins::array_element_type(
        &element_source,
    )];
    if matches!(cx.args[1].kind, crate::parser::ast::ExprKind::Closure { .. }) {
        callback_arg_types.push(crate::types::checker::builtins::array_key_type(&element_source));
    }
    let label = format!("{}() callback", cx.name);
    crate::types::checker::builtins::check_array_callback_builtin_call(
        cx.checker,
        &cx.args[1],
        &callback_arg_types,
        cx.span,
        cx.env,
        &label,
    )?;
    Ok(PhpType::Bool)
}
