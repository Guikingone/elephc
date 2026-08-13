//! Purpose:
//! Home of the PHP `join` builtin: the registry-visible alias of `implode`.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - `join` shares `implode`'s typed runtime target (`RuntimeFnId::Implode`), so the
//!   registry's alias arity gate requires the two declarations to agree on their
//!   enforced bounds: one required parameter, at most two.
//! - Reference PHP declares `join(string|array $separator, ?array $array = null)`, which
//!   is what makes BOTH accepted call forms work: `join($separator, $array)` and the
//!   single-argument `join($array)` that joins with an empty separator. `separator` is
//!   therefore declared `Mixed` here, not `Str`, and the backend picks the operand roles
//!   from the argument count. (The legacy reversed `join($array, $separator)` order was
//!   REMOVED in PHP 8.0 and is deliberately not accepted.)
//! - Unlike `implode`, no `check` hook narrows the arity: the one-argument form is the
//!   whole reason this alias carries its own declaration.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "join",
    area: String,
    params: [separator: Mixed, array: Mixed = DefaultSpec::Null],
    max_args: 2,
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Implode,
    ),
    summary: "Joins array elements into a single string using a separator (alias of implode).",
    php_manual: "https://www.php.net/manual/en/function.join.php",
}
