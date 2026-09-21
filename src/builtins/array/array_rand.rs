//! Purpose:
//! Home of the PHP `array_rand` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` validates the argument is an array and returns `Int` (the randomly
//!   selected integer index). The declared `returns: Mixed` is the FCC type.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

builtin! {
    contract: "array_rand",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayRand,
    ),
}

/// Validates that the argument is an array and returns `Int`.
///
/// The registry's `check_arity` handles arity enforcement (exactly 1 argument).
/// The runtime always returns a single random integer index from the array.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    if !crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(&ty) {
        return Err(CompileError::new(
            cx.span,
            "array_rand() argument must be array",
        ));
    }
    // Anything that is not a dense indexed array answers through
    // `__elephc_array_rand_gradual`, whose key can be a string as easily as an integer.
    let gradual = !matches!(ty, PhpType::Array(_));
    // `array_rand($a, 1)` IS `array_rand($a)`: php-src returns one key for both and only widens
    // to an array of keys when `$num` is greater than one. Twig's `CoreExtension::random()` writes
    // the explicit spelling, which cost the whole file. Any other `$num` still has no lowering.
    if let Some(num) = cx.args.get(1) {
        cx.checker.infer_type(num, cx.env)?;
        if !matches!(num.kind, ExprKind::IntLiteral(1)) {
            return Err(CompileError::new(
                cx.span,
                "array_rand() num argument must be the literal 1 in AOT mode",
            ));
        }
    }
    if gradual {
        return Ok(PhpType::Mixed);
    }
    Ok(PhpType::Int)
}
