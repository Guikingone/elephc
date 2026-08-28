//! Purpose:
//! Home of the PHP `base64_decode` builtin and its backend-neutral runtime semantics.
//!
//! Called from:
//! - The builtin registry, checker, optimizer, and AST-to-EIR builtin lowering path.
//!
//! Key details:
//! - The declared signature is PHP's own
//!   `base64_decode(string $string, bool $strict = false): string|false`. The `$strict` flag
//!   is what makes the result a union: reference PHP returns `false` from a strict decode
//!   whose input holds a character outside the Base64 alphabet, a misplaced `=`, or a
//!   truncated final group.
//! - `check` returns `string|false`, whose codegen representation is `Mixed`, so the backend
//!   hands back a BOXED cell (the `strstr()` / `phpversion($ext)` shape) instead of a raw
//!   string-register pair.
//! - Decoding itself follows php-src's `php_base64_decode_impl`: a per-character reverse
//!   table where whitespace is skipped in both modes, an `i % 4` accumulator that does not
//!   restart on skipped bytes, and PHP's padding rules. `_b64_decode_tbl` carries the
//!   sentinels that distinguish "skip" from "reject".

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "base64_decode",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Base64Decode,
    ),
}

/// Returns `string|false` for every `base64_decode()` call.
///
/// ONE type for every arity, deliberately. A one-argument (lax) call can never fail, but the
/// checker-facing type and the backend's storage layout are a single shared contract: the
/// lowering always boxes its answer into a `Mixed` cell because the two-argument form may
/// yield `false`, and narrowing the lax arity back to `Str` here would make `store_if_result`
/// copy the string-pair registers that no longer hold the answer. `strstr()` documents the
/// same reasoning and the miscompile that disagreement causes.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(cx
        .checker
        .normalize_union_type(vec![PhpType::Str, PhpType::False]))
}
