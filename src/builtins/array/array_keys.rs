//! Purpose:
//! Home of the PHP `array_keys` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` reproduces the legacy return-type rule: an indexed array of a known
//!   element type yields `Array<Int>` (positional keys) while an associative array
//!   yields `Array<key>`. A gradual array boundary — including `array<mixed>`, whose
//!   runtime payload may be a string-keyed hash — yields `Array<Mixed>` because its
//!   runtime key shape is unknown. A check hook is required because the return type
//!   depends on the inferred argument type, which the `builtin!` `returns:` field
//!   cannot express.
//! - The hook's answer is what the EIR backend consumes: a `check` hook flips the
//!   descriptor's result type to `Checked` (`with_registry_checker_contract`), so
//!   `registry_builtin_result_type` reads the checker's per-span record. The hook and
//!   `crate::codegen::lower_inst::builtins::arrays::keys` must therefore agree on the
//!   key shape for every argument type, or the build fails on the disagreement.
//! - Arity (exactly 1 argument) is validated by the registry's `check_arity` before
//!   the hook fires; the inline arity check from the legacy arm is not reproduced here.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "array_keys",
    area: Array,
    params: [array: Mixed],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayKeys,
    ),
    summary: "Returns all the keys of an array.",
    php_manual: "https://www.php.net/manual/en/function.array-keys.php",
}

/// Returns the key-array type for an `array_keys` call.
///
/// An indexed array of a *known* element type produces `Array<Int>`; an associative
/// array produces `Array<key>`. `Mixed` and unions containing an array produce
/// `Array<Mixed>` and defer the concrete check to the EIR runtime boundary. Other
/// argument types are rejected. The argument is re-inferred here to drive the return
/// type; the registry already inferred it once for side effects, and arity is
/// pre-validated.
///
/// `array<mixed>` — what a bare `array` type hint and a heterogeneous literal both
/// become — is elephc's *gradual* array: it asserts nothing about the key shape, and
/// its runtime payload may be an insertion-ordered hash with string keys. The EIR
/// backend already knows this and resolves the storage kind at runtime
/// (`lower_dynamic_mixed_array_keys` branches on `__rt_heap_kind`), so its keys are
/// genuinely `int|string`. Answering `Array<Int>` there made the two layers disagree
/// on the same call: the backend refused outright ("array_keys associative key PHP
/// type Mixed into result PHP type Int") rather than truncating string keys into
/// integer slots. The predicate below is deliberately the same one the backend
/// dispatches on, so the two cannot drift apart again.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    match ty {
        PhpType::Array(elem) if elem.codegen_repr() == PhpType::Mixed => {
            Ok(PhpType::Array(Box::new(PhpType::Mixed)))
        }
        PhpType::Array(_) => Ok(PhpType::Array(Box::new(PhpType::Int))),
        PhpType::AssocArray { key, .. } => Ok(PhpType::Array(key)),
        t if crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(&t) => {
            Ok(PhpType::Array(Box::new(PhpType::Mixed)))
        }
        _ => Err(CompileError::new(
            cx.span,
            "array_keys() argument must be array",
        )),
    }
}
