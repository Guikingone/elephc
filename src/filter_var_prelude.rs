//! Purpose:
//! Injects the elephc-PHP `__elephc_filter_var_dyn()` helper that implements `filter_var()` when
//! the `$filter` argument is a runtime (non-compile-time-constant) integer. `filter_var()` with a
//! literal filter id is lowered directly (see `crate::ir_lower::expr::filter`); Symfony's
//! `InputBag`/`ParameterBag::filter()` pass a VARIABLE `$filter`, which the checker used to reject.
//! The dynamic call is routed to this helper by `crate::ir_lower::expr::lower_function_call`.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject_if_used`, at the same
//!   pipeline stage as `crate::var_export_prelude` (after `autoload::run` and the conditional-
//!   function hoist, before the type checker collects functions).
//! - `crate::ir_lower::expr::filter::lower_static_filter_var` calls the helper by its bare symbol
//!   name (`DYN_FILTER_VAR_NAME`) for every dynamic-`$filter` call site.
//!
//! Key details:
//! - The helper dispatches the runtime `$filter` id to the SAME per-filter behavior the literal
//!   path already implements, by delegating to `filter_var(..., <LITERAL id>, <LITERAL flags>)`
//!   inside each branch — so the runtime dispatch reuses the tested, target-aware
//!   `FILTER_VALIDATE_INT/FLOAT/BOOL/IP` and `FILTER_DEFAULT` codegen with no new assembly.
//! - `FILTER_NULL_ON_FAILURE` is honored generally: it only changes the failure sentinel
//!   (`false` -> `null`), and each branch selects the matching literal `_nof` variant, so a valid
//!   `false` (e.g. `FILTER_VALIDATE_BOOL` on `"off"`) is never mis-mapped to `null`.
//! - `$options` may be an int (flags) or the array form `['flags' => X]`, matching PHP. An int
//!   (or int-`mixed`) options argument is routed to `__elephc_filter_var_dyn`; a statically
//!   array-typed options argument is routed to `__elephc_filter_var_dyn_arr` (an `array`-typed
//!   parameter, avoiding the `mixed`-boxed-array seam). A `mixed`-typed options that holds an
//!   array at runtime is NOT handled (pre-existing elephc Mixed-array-element-access gap) — a
//!   documented residual, see `crate::ir_lower::expr::filter`. `FILTER_FLAG_IPV4`/`IPV6` ARE
//!   honored for `FILTER_VALIDATE_IP`; the `'options'` sub-array (min_range/max_range/regexp) and
//!   REQUIRE_ARRAY/FORCE_ARRAY are NOT honored in the dynamic path (the literal path rejects them
//!   too).
//! - Filters elephc does not implement (EMAIL/URL/DOMAIN/MAC/REGEXP/SANITIZE_*) or a genuinely
//!   unknown id emit a LOUD `error_log` warning (the EIR backend does not lower `trigger_error`)
//!   and fail closed (`false`/`null`) rather than silently mis-validating — the literal path
//!   rejects those ids too, so this is a documented, honest gap.

use crate::parser::ast::Program;

/// The bare symbol name of the int-flags dynamic-`$filter` helper. Kept as a constant so the
/// prelude source and the `crate::ir_lower::expr::filter` call site cannot drift apart. The
/// `__elephc_` prefix is reserved, so this never collides with user code.
pub(crate) const DYN_FILTER_VAR_NAME: &str = "__elephc_filter_var_dyn";

/// The bare symbol name of the array-`$options` dynamic-`$filter` helper. Used when the routed
/// options argument is statically array-typed, so the array is passed to an `array`-typed
/// parameter (NOT a `mixed` one): an array boxed into `mixed` is a pre-existing elephc gap
/// (element access fatals at runtime), so the array must never take the `mixed` seam.
pub(crate) const DYN_FILTER_VAR_ARR_NAME: &str = "__elephc_filter_var_dyn_arr";

/// The elephc-PHP dynamic-`filter_var` prelude: the int-flags `__elephc_filter_var_dyn` helper
/// plus the array-options `__elephc_filter_var_dyn_arr` shim. Each branch delegates to
/// `filter_var()` with a LITERAL filter id and flags, so the existing literal lowering handles the
/// real work.
pub const FILTER_VAR_DYN_PRELUDE_SRC: &str = r#"<?php
function __elephc_filter_var_dyn_arr(mixed $value, int $filter, array $options): mixed {
    $flags = isset($options['flags']) ? (int) $options['flags'] : 0;
    return __elephc_filter_var_dyn($value, $filter, $flags);
}
function __elephc_filter_var_dyn(mixed $value, int $filter, mixed $flags = 0): mixed {
    if (is_array($flags)) {
        // A `mixed`-typed `$options` that holds an array cannot be read here (pre-existing elephc
        // Mixed-array-element-access gap: element access/iteration on a `mixed`-boxed array fatals).
        // Fail LOUD rather than silently mis-handle the flags — a statically array-typed options
        // argument is instead routed to `__elephc_filter_var_dyn_arr`, which reads it correctly.
        error_log("filter_var(): array \$options passed through a mixed value is not supported by elephc yet; flags ignored");
        $flags = 0;
    } else {
        $flags = (int) $flags;
    }
    $nof = ($flags & FILTER_NULL_ON_FAILURE) !== 0;
    if ($filter === FILTER_VALIDATE_INT) {
        return $nof
            ? filter_var($value, FILTER_VALIDATE_INT, FILTER_NULL_ON_FAILURE)
            : filter_var($value, FILTER_VALIDATE_INT);
    }
    if ($filter === FILTER_VALIDATE_FLOAT) {
        return $nof
            ? filter_var($value, FILTER_VALIDATE_FLOAT, FILTER_NULL_ON_FAILURE)
            : filter_var($value, FILTER_VALIDATE_FLOAT);
    }
    if ($filter === FILTER_VALIDATE_BOOL) {
        return $nof
            ? filter_var($value, FILTER_VALIDATE_BOOL, FILTER_NULL_ON_FAILURE)
            : filter_var($value, FILTER_VALIDATE_BOOL);
    }
    if ($filter === FILTER_VALIDATE_IP) {
        $v4 = ($flags & FILTER_FLAG_IPV4) !== 0;
        $v6 = ($flags & FILTER_FLAG_IPV6) !== 0;
        if ($v4 && !$v6) {
            return $nof
                ? filter_var($value, FILTER_VALIDATE_IP, FILTER_FLAG_IPV4 | FILTER_NULL_ON_FAILURE)
                : filter_var($value, FILTER_VALIDATE_IP, FILTER_FLAG_IPV4);
        }
        if ($v6 && !$v4) {
            return $nof
                ? filter_var($value, FILTER_VALIDATE_IP, FILTER_FLAG_IPV6 | FILTER_NULL_ON_FAILURE)
                : filter_var($value, FILTER_VALIDATE_IP, FILTER_FLAG_IPV6);
        }
        return $nof
            ? filter_var($value, FILTER_VALIDATE_IP, FILTER_NULL_ON_FAILURE)
            : filter_var($value, FILTER_VALIDATE_IP);
    }
    if ($filter === FILTER_DEFAULT) {
        return $nof
            ? filter_var($value, FILTER_DEFAULT, FILTER_NULL_ON_FAILURE)
            : filter_var($value, FILTER_DEFAULT);
    }
    error_log("filter_var(): filter with ID " . $filter . " is not supported by elephc yet");
    return $nof ? null : false;
}
"#;

/// Prepends the dynamic-`filter_var` prelude when the program references `filter_var`, so binaries
/// without any `filter_var` usage carry nothing. Injecting on ANY `filter_var` reference (not only
/// dynamic ones) is sound: a program whose `filter_var` calls are all literal never calls the
/// helper, so it is dead code; a program with a dynamic call has `filter_var` referenced and thus
/// gets the helper it needs. The source is static and tested, so a tokenize/parse failure is a
/// compiler bug and panics rather than degrading silently.
pub fn inject_if_used(program: Program) -> Program {
    if !crate::ast_usage::collect(&program).references("filter_var") {
        return program;
    }
    let tokens = crate::lexer::tokenize(FILTER_VAR_DYN_PRELUDE_SRC)
        .expect("filter_var dynamic prelude must tokenize");
    let mut combined = crate::parser::parse(&tokens).expect("filter_var dynamic prelude must parse");
    combined.extend(program);
    combined
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Function-level tests for the `inject_if_used` pay-for-use guard.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Source is parsed the way `inject_if_used` sees it: tokenize then parse.

    use super::*;

    /// Parses source the way `inject_if_used` sees it: tokenize then parse.
    fn parse(source: &str) -> Program {
        let tokens = crate::lexer::tokenize(source).expect("test source must tokenize");
        crate::parser::parse(&tokens).expect("test source must parse")
    }

    /// A program with no `filter_var` usage is returned unchanged.
    #[test]
    fn no_injection_when_unused() {
        let program = parse(r#"<?php $a = [1, 2]; echo count($a);"#);
        let injected = inject_if_used(program.clone());
        assert_eq!(injected.len(), program.len());
    }

    /// A program that calls `filter_var` gets the helper prelude prepended.
    #[test]
    fn injection_when_used() {
        let program = parse(r#"<?php $f = FILTER_VALIDATE_INT; $x = filter_var("5", $f);"#);
        let injected = inject_if_used(program.clone());
        assert!(injected.len() > program.len());
    }
}
