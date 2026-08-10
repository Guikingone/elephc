//! Purpose:
//! Injects an elephc-PHP implementation of gradual `array_reduce()` forms.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness before type checking.
//!
//! Key details:
//! - PHP defaults the missing initial carry to `null`; the native runtime reducer currently
//!   requires an explicit third operand and has an integer accumulator ABI.
//! - The PHP helper keeps the carry boxed as `mixed`, so callbacks that build strings or return
//!   other gradual values preserve their real result instead of being forced through an integer.

use crate::parser::ast::Program;

/// Reserved function name used for gradual `array_reduce()` calls.
pub(crate) const ARRAY_REDUCE_DEFAULT_NAME: &str = "__elephc_array_reduce_default";

/// PHP source for the injected boxed-carry `array_reduce()` implementation.
pub const ARRAY_REDUCE_PRELUDE_SRC: &str = r#"<?php
function __elephc_array_reduce_default(mixed $array, callable $callback, mixed $initial = null): mixed {
    $carry = $initial;
    foreach ($array as $value) {
        $carry = $callback($carry, $value);
    }
    return $carry;
}
"#;

/// Prepends the helper when the resolved program references `array_reduce`.
pub fn inject_if_used(program: Program) -> Program {
    if !crate::ast_usage::collect(&program).references("array_reduce") {
        return program;
    }
    let tokens = crate::lexer::tokenize(ARRAY_REDUCE_PRELUDE_SRC)
        .expect("array_reduce prelude must tokenize");
    let mut combined = crate::parser::parse(&tokens).expect("array_reduce prelude must parse");
    combined.extend(program);
    combined
}
