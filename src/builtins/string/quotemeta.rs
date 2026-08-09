//! Purpose:
//! Home of the PHP `quotemeta` builtin and its backend-neutral runtime semantics.
//!
//! Called from:
//! - The builtin registry, checker, optimizer, and AST-to-EIR builtin lowering path.
//!
//! Key details:
//! - The typed runtime target has a validated `Str -> Str` EIR signature.
//! - Escaping never inspects state outside its argument, so the call is `PURE`.
//! - Concrete helper symbols and registers are selected only by the target backend.

use crate::ir::{RuntimeCallTarget, UnaryStringRuntime};

builtin! {
    name: "quotemeta",
    area: String,
    params: [string: Str],
    returns: Str,
    semantics: crate::builtins::semantics::unary_string_runtime(
        RuntimeCallTarget::UnaryString(UnaryStringRuntime::QuoteMeta),
        crate::ir::Effects::PURE,
    ),
    summary: "Prefixes each regular-expression metacharacter in a string with a backslash.",
    php_manual: "https://www.php.net/manual/en/function.quotemeta.php",
}
