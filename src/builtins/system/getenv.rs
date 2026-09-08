//! Purpose:
//! Home of the PHP `getenv` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` returns `Union(Str, False)` when a name is present, matching PHP's
//!   string-or-false lookup. An omitted or null name answers the whole environment
//!   as a boxed Mixed hash — including `getenv(null)`, `getenv(null, true)`, and
//!   `getenv(local_only: true)`.
//! - The named-lookup EIR result carries that union too. It used to be overridden
//!   to plain `Str` "for present and missing variables alike", which is where the
//!   two answers were collapsed: an unset variable came back as `""`, so
//!   `getenv($x) !== false` — the idiom for "is this set" — was true for every
//!   name, silently.

use crate::builtins::semantics::{
    runtime_fn_semantics, BuiltinResultType, BuiltinSemanticInput, BuiltinSemantics,
};
use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::names::php_symbol_key;
use crate::parser::ast::{Expr, ExprKind};
use crate::types::PhpType;

builtin! {
    contract: "getenv",
    check: check,
    semantics: getenv_semantics(),
}

/// Builds semantics whose EIR result type follows the name, not the physical arity.
const fn getenv_semantics() -> BuiltinSemantics {
    let mut semantics = runtime_fn_semantics(crate::ir::RuntimeFnId::Getenv);
    semantics.result_type = BuiltinResultType::Shared(eir_result_type);
    semantics
}

/// Returns Mixed for the whole-environment form and `string|false` for a named lookup.
///
/// ARITY IS READ FROM `arg_types`, NOT `args`: `semantics::lower_registry_call` re-resolves
/// this hook with `args: &[]` while `ir_lower` resolves it with the real AST args.
/// `arg_types` is derived from the lowered operands in both paths. A null name is still
/// an operand, so keying off `args.is_empty()` would type `getenv(null)` as a string
/// lookup — the same mistake as selecting `__rt_getenv` from a non-empty operand list.
fn eir_result_type(input: &BuiltinSemanticInput<'_>) -> PhpType {
    if getenv_arg_types_select_whole_environment(input.arg_types) {
        PhpType::Mixed
    } else {
        PhpType::Union(vec![PhpType::Str, PhpType::False])
    }
}

/// True when there is no name, or the name is the null default.
fn getenv_arg_types_select_whole_environment(arg_types: &[PhpType]) -> bool {
    match arg_types.first() {
        None => true,
        Some(ty) => matches!(ty, PhpType::Void),
    }
}

/// Returns the type PHP's signature declares, which depends on the NAME, not just arity.
///
/// `getenv($name)` answers `string|false`. `getenv()`, `getenv(null)`, `getenv(null, true)`,
/// and `getenv(local_only: true)` answer the whole environment as a string-keyed array,
/// and cannot fail — there is no name to miss. Returning the union for those would make
/// every caller handle a `false` that cannot occur, and `foreach (getenv() as ...)` would
/// not type-check.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    for arg in cx.args.iter() {
        cx.checker.infer_type(arg, cx.env)?;
    }
    if getenv_ast_selects_whole_environment(cx)? {
        // `Mixed`, not `AssocArray`: the result is a hash pointer BOXED in a Mixed
        // cell, exactly as `getdate` and `stat` return theirs, and those declare
        // `Mixed` for that reason. Declaring the array type instead tells every
        // consumer the value IS the hash, so `count()` reads the cell's tag as
        // the entry count and answers 5 for a 65-entry environment.
        //
        // The type is the REPRESENTATION, and losing `array<string,string>` here
        // is the price of the box.
        return Ok(PhpType::Mixed);
    }
    Ok(cx.checker.normalize_union_type(vec![PhpType::Str, PhpType::False]))
}

/// True when the call names no variable, or names null.
fn getenv_ast_selects_whole_environment(
    cx: &mut BuiltinCheckCtx<'_>,
) -> Result<bool, CompileError> {
    match getenv_name_argument(cx.args) {
        None => Ok(true),
        Some(name) => {
            let ty = cx.checker.infer_type(name, cx.env)?;
            Ok(matches!(ty, PhpType::Void))
        }
    }
}

/// Returns the `$name` expression, or `None` when the name was omitted.
///
/// `getenv(local_only: true)` supplies only the flag; the name stays at its null
/// default and must still select the whole-environment form.
fn getenv_name_argument(args: &[Expr]) -> Option<&Expr> {
    let mut named_name = None;
    let mut first_positional = None;
    for arg in args {
        match &arg.kind {
            ExprKind::NamedArg { name, value } => {
                if php_symbol_key(name) == "name" {
                    named_name = Some(value.as_ref());
                }
            }
            _ if first_positional.is_none() => first_positional = Some(arg),
            _ => {}
        }
    }
    named_name.or(first_positional)
}
