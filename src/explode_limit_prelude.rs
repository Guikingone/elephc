//! Purpose:
//! Injects a target-independent elephc-PHP implementation of `explode()`'s `$limit` argument.
//! The two-argument form keeps its native splitter; only a three-argument call is redirected here.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness before type checking.
//!
//! Key details:
//! - Three-argument `explode()` calls are name-resolved to a reserved internal alias before injection.
//! - A positive limit merges the trailing segments; a negative one drops them; zero behaves as one.

use crate::parser::ast::Program;

/// Reserved function name used for the PHP implementation of three-argument `explode()` calls.
pub(crate) const EXPLODE_LIMIT_FUNCTION_NAME: &str = "__elephc_explode_limit";

/// PHP source for the injected `explode($separator, $string, $limit)` implementation.
///
/// Built entirely on the two-argument `explode()` (which lowers to the native `__rt_explode`
/// splitter) plus `array_slice`/`implode`, rather than on a second hand-written assembly splitter
/// per architecture. Re-joining the tail with `$separator` reconstructs the original suffix exactly,
/// because those segments came from splitting on that same separator.
pub const EXPLODE_LIMIT_PRELUDE_SRC: &str = r#"<?php
function __elephc_explode_limit(string $separator, string $string, int $limit): array {
    if ($limit === 0) {
        return [$string];
    }

    $parts = explode($separator, $string);
    $count = count($parts);

    if ($limit > 0) {
        if ($limit >= $count) {
            return $parts;
        }
        $head = array_slice($parts, 0, $limit - 1);
        $head[] = implode($separator, array_slice($parts, $limit - 1));
        return $head;
    }

    $keep = $count + $limit;
    if ($keep <= 0) {
        return [];
    }
    return array_slice($parts, 0, $keep);
}
"#;

/// Prepends the three-argument `explode()` prelude when its resolved internal alias is referenced.
pub fn inject_if_used(program: Program) -> Program {
    if !crate::ast_usage::collect(&program).references(EXPLODE_LIMIT_FUNCTION_NAME) {
        return program;
    }
    let tokens = crate::lexer::tokenize(EXPLODE_LIMIT_PRELUDE_SRC)
        .expect("explode limit prelude must tokenize");
    let mut combined =
        crate::parser::parse(&tokens).expect("explode limit prelude must parse");
    combined.extend(program);
    combined
}
