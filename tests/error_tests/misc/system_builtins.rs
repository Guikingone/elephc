//! Purpose:
//! Integration or regression tests for diagnostic coverage of misc system builtin diagnostics, including undefined constant, define wrong args, and define non string name.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Invalid PHP snippets are checked through shared diagnostic helpers for messages, spans, and recovery behavior.

use super::*;

expect_builtin_arity_error!(
    test_error_exit_wrong_args,
    "<?php exit(1, 2);",
    "exit() takes 0 or 1 arguments"
);

expect_builtin_arity_error!(
    test_error_die_wrong_args,
    "<?php die(1, 2);",
    "exit() takes 0 or 1 arguments"
);

expect_builtin_arity_error!(
    test_error_serialize_no_args,
    "<?php serialize();",
    "serialize() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_serialize_too_many_args,
    "<?php $a = 1; $b = 2; serialize($a, $b);",
    "serialize() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_unserialize_no_args,
    "<?php unserialize();",
    "unserialize() takes 1 or 2 arguments"
);

expect_builtin_arity_error!(
    test_error_unserialize_too_many_args,
    "<?php $a = \"x\"; unserialize($a, [], 3);",
    "unserialize() takes 1 or 2 arguments"
);

expect_builtin_arity_error!(
    test_error_set_time_limit_no_args,
    "<?php set_time_limit();",
    "set_time_limit() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_connection_aborted_too_many_args,
    "<?php connection_aborted(1);",
    "connection_aborted() takes no arguments"
);

expect_builtin_arity_error!(
    test_error_error_reporting_too_many_args,
    "<?php error_reporting(1, 2);",
    "error_reporting() takes at most 1 argument"
);

expect_builtin_arity_error!(
    test_error_gc_enabled_too_many_args,
    "<?php gc_enabled(1);",
    "gc_enabled() takes no arguments"
);

expect_builtin_arity_error!(
    test_error_gc_enable_too_many_args,
    "<?php gc_enable(1);",
    "gc_enable() takes no arguments"
);

expect_builtin_arity_error!(
    test_error_gc_disable_too_many_args,
    "<?php gc_disable(1);",
    "gc_disable() takes no arguments"
);

expect_builtin_arity_error!(
    test_error_gc_collect_cycles_too_many_args,
    "<?php gc_collect_cycles(1);",
    "gc_collect_cycles() takes no arguments"
);

expect_builtin_arity_error!(
    test_error_gc_mem_caches_too_many_args,
    "<?php gc_mem_caches(1);",
    "gc_mem_caches() takes no arguments"
);

expect_builtin_arity_error!(
    test_error_memory_get_usage_too_many_args,
    "<?php memory_get_usage(true, false);",
    "memory_get_usage() takes at most 1 argument"
);

expect_builtin_arity_error!(
    test_error_error_get_last_too_many_args,
    "<?php error_get_last(1);",
    "error_get_last() takes no arguments"
);

expect_builtin_arity_error!(
    test_error_libxml_use_internal_errors_too_many_args,
    "<?php libxml_use_internal_errors(true, false);",
    "libxml_use_internal_errors() takes at most 1 argument"
);

expect_builtin_arity_error!(
    test_error_libxml_clear_errors_too_many_args,
    "<?php libxml_clear_errors(1);",
    "libxml_clear_errors() takes no arguments"
);

expect_builtin_arity_error!(
    test_error_libxml_get_errors_too_many_args,
    "<?php libxml_get_errors(1);",
    "libxml_get_errors() takes no arguments"
);

expect_builtin_arity_error!(
    test_error_eval_wrong_args,
    "<?php eval();",
    "eval() takes exactly 1 argument"
);

/// Verifies an eval barrier allows later reads of variables that eval may create dynamically.
#[test]
fn test_eval_barrier_allows_dynamic_variable_read() {
    check_source("<?php eval('$created = 1;'); echo $created;")
        .expect("eval-created variable reads after the barrier should type-check");
}

/// Verifies an eval barrier allows later calls to functions that eval may declare dynamically.
#[test]
fn test_eval_barrier_allows_dynamic_function_call() {
    check_source("<?php eval('function dyn_eval_error_test() { return 1; }'); echo dyn_eval_error_test();")
        .expect("eval-declared function calls after the barrier should type-check");
}

/// Verifies eval does not hide undefined-variable reads that happen before the barrier.
#[test]
fn test_eval_barrier_does_not_hide_prior_undefined_variable() {
    expect_error(
        "<?php echo $created; eval('$created = 1;');",
        "Undefined variable: $created",
    );
}

/// Verifies that referencing an undefined constant produces the expected "Undefined constant" error.
#[test]
fn test_error_undefined_constant() {
    expect_error("<?php echo UNDEFINED_CONST;", "Undefined constant");
}

/// Verifies that an unqualified reference to a genuinely-undefined constant
/// inside a namespace still errors with the namespaced FQN. Guards that the
/// name_resolver's new `define()` symbol collection does not over-accept: only
/// constants actually created by a `define('LITERAL', ...)` call are registered.
#[test]
fn test_error_namespace_undefined_constant_no_define() {
    expect_error(
        "<?php namespace Demo\\Missing; echo MISSING_CONST;",
        "Undefined constant: Demo\\Missing\\MISSING_CONST",
    );
}

/// Verifies that `define()` with a single argument (missing value) yields a wrong-args diagnostic.
#[test]
fn test_error_define_wrong_args() {
    expect_error("<?php define(\"X\");", "define() takes exactly 2 arguments");
}

/// Verifies that `define()` with a non-string first argument (int name) yields a non-string-name error.
#[test]
fn test_error_define_non_string_name() {
    expect_error(
        "<?php define(42, 100);",
        "define() first argument must be a string literal",
    );
}

/// Verifies that `defined()` requires exactly one argument.
#[test]
fn test_error_defined_wrong_args() {
    expect_error("<?php defined();", "defined() takes exactly 1 argument");
}

/// Verifies that `constant()` requires exactly one argument. A non-literal name
/// is intentionally accepted (it lowers to the runtime constant registry), so the
/// remaining compile-time error surface is the argument count.
#[test]
fn test_error_constant_wrong_args() {
    expect_error(
        "<?php constant(\"A\", \"B\");",
        "constant() takes exactly 1 argument",
    );
}

// -- List unpack errors --

/// Verifies that `time()` with any arguments yields a no-args diagnostic.
#[test]
fn test_error_time_wrong_args() {
    expect_error("<?php time(1);", "time() takes no arguments");
}

/// Verifies that `microtime()` with two arguments yields a wrong-args diagnostic.
#[test]
fn test_error_microtime_wrong_args() {
    expect_error(
        "<?php microtime(1, 2);",
        "microtime() takes 0 or 1 arguments",
    );
}

/// Verifies that `sleep()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_sleep_wrong_args() {
    expect_error("<?php sleep();", "sleep() takes exactly 1 argument");
}

/// Verifies that `usleep()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_usleep_wrong_args() {
    expect_error("<?php usleep();", "usleep() takes exactly 1 argument");
}

/// Verifies that `getenv()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_getenv_wrong_args() {
    expect_error("<?php getenv();", "getenv() takes exactly 1 argument");
}

/// Verifies that `putenv()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_putenv_wrong_args() {
    expect_error("<?php putenv();", "putenv() takes exactly 1 argument");
}

/// Verifies that `phpversion()` with more than one argument yields an arity diagnostic.
///
/// The zero-argument (version string) and one-argument (`?string $extension`,
/// always false in elephc) forms are both valid, so only two or more arguments
/// stay loud.
#[test]
fn test_error_phpversion_wrong_args() {
    expect_error(
        "<?php phpversion(1, 2);",
        "phpversion() takes at most 1 argument",
    );
}

/// Verifies that `extension_loaded()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_extension_loaded_wrong_args() {
    expect_error(
        "<?php extension_loaded();",
        "extension_loaded() takes exactly 1 argument",
    );
}

/// Verifies that `php_uname()` with two arguments yields a wrong-args diagnostic.
#[test]
fn test_error_php_uname_wrong_args() {
    expect_error(
        "<?php php_uname(1, 2);",
        "php_uname() takes 0 or 1 arguments",
    );
}

/// Verifies that `php_uname()` with a non-string mode argument yields a wrong-type diagnostic.
#[test]
fn test_error_php_uname_wrong_type() {
    expect_error("<?php php_uname(1);", "php_uname() argument must be string");
}

/// Verifies that `exec()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_exec_wrong_args() {
    expect_error("<?php exec();", "exec() takes exactly 1 argument");
}

/// Verifies that `shell_exec()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_shell_exec_wrong_args() {
    expect_error(
        "<?php shell_exec();",
        "shell_exec() takes exactly 1 argument",
    );
}

/// Verifies that `system()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_system_wrong_args() {
    expect_error("<?php system();", "system() takes exactly 1 argument");
}

/// Verifies that `passthru()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_passthru_wrong_args() {
    expect_error("<?php passthru();", "passthru() takes exactly 1 argument");
}

// -- Global/Static parse errors --

/// Verifies that `date()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_date_no_args() {
    expect_error("<?php date();", "date() takes 1 or 2 arguments");
}

/// Verifies that `gmdate()` with no arguments yields a wrong-args diagnostic naming `gmdate`.
#[test]
fn test_error_gmdate_no_args() {
    expect_error("<?php gmdate();", "gmdate() takes 1 or 2 arguments");
}

/// Verifies that `gmdate()` with three arguments yields a wrong-args diagnostic.
#[test]
fn test_error_gmdate_too_many_args() {
    expect_error(
        "<?php gmdate(\"Y\", 0, 1);",
        "gmdate() takes 1 or 2 arguments",
    );
}

/// Verifies `mktime()` arity: PHP 8.0+ accepts 0–6 arguments (omitted ones default to the
/// corresponding current-time component via the procedural-alias desugar), so seven arguments is
/// out of range and yields the fixed-arity diagnostic.
#[test]
fn test_error_mktime_wrong_args() {
    expect_error(
        "<?php mktime(1, 2, 3, 4, 5, 6, 7);",
        "mktime() takes exactly 6 arguments",
    );
}

/// Verifies that `gmmktime()` rejects more than six arguments, mirroring `mktime()`.
#[test]
fn test_error_gmmktime_wrong_args() {
    expect_error(
        "<?php gmmktime(1, 2, 3, 4, 5, 6, 7);",
        "gmmktime() takes exactly 6 arguments",
    );
}

/// Verifies that `getdate()` rejects a second argument (it accepts at most one).
#[test]
fn test_error_getdate_wrong_args() {
    expect_error(
        "<?php getdate(1, 2);",
        "getdate() takes at most 1 argument",
    );
}

/// Verifies that `localtime()` rejects a third argument (it accepts at most two).
#[test]
fn test_error_localtime_wrong_args() {
    expect_error(
        "<?php localtime(1, true, 3);",
        "localtime() takes at most 2 arguments",
    );
}

/// Verifies that `date_default_timezone_get()` rejects any argument.
#[test]
fn test_error_date_default_timezone_get_wrong_args() {
    expect_error(
        "<?php date_default_timezone_get(\"x\");",
        "date_default_timezone_get() takes no arguments",
    );
}

/// Verifies that `date_default_timezone_set()` requires exactly one argument.
#[test]
fn test_error_date_default_timezone_set_wrong_args() {
    expect_error(
        "<?php date_default_timezone_set();",
        "date_default_timezone_set() takes exactly 1 argument",
    );
}

/// Verifies that `strtotime()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_strtotime_no_args() {
    expect_error("<?php strtotime();", "strtotime() takes 1 or 2 arguments");
}

/// Verifies that `strtotime()` with three arguments yields a wrong-args diagnostic
/// (the optional `baseTimestamp` is the only second argument).
#[test]
fn test_error_strtotime_too_many_args() {
    expect_error(
        "<?php strtotime(\"now\", 0, 1);",
        "strtotime() takes 1 or 2 arguments",
    );
}

/// Verifies that `checkdate()` with two arguments yields a wrong-args diagnostic
/// (it requires exactly month, day, and year).
#[test]
fn test_error_checkdate_wrong_args() {
    expect_error(
        "<?php checkdate(1, 2);",
        "checkdate() takes exactly 3 arguments",
    );
}

// -- date/time alias arity diagnostics --
// Procedural date/time aliases are desugared by the name resolver only at their supported
// arities. A wrong-arity call must report a precise arity error (matching `function_exists()`,
// which recognizes these names) rather than the misleading "Undefined function". Each test below
// covers a distinct message shape produced by the checker's alias-arity diagnostic.

/// `idate()` accepts 1 or 2 arguments; a zero-arg call reports the "N or M" message shape.
#[test]
fn test_error_idate_too_few_args() {
    expect_error("<?php idate();", "idate() takes 1 or 2 arguments");
}

/// A date alias called with too MANY arguments is diagnosed by arity, not as undefined.
#[test]
fn test_error_idate_too_many_args() {
    expect_error("<?php idate(\"Y\", 0, 1);", "idate() takes 1 or 2 arguments");
}

/// `gregoriantojd()` requires exactly 3 arguments (the "exactly N" message shape).
#[test]
fn test_error_gregoriantojd_wrong_args() {
    expect_error(
        "<?php gregoriantojd(1, 2);",
        "gregoriantojd() takes exactly 3 arguments",
    );
}

/// `jdtogregorian()` requires exactly 1 argument (singular wording in the message).
#[test]
fn test_error_jdtogregorian_wrong_args() {
    expect_error(
        "<?php jdtogregorian();",
        "jdtogregorian() takes exactly 1 argument",
    );
}

/// `easter_date()` accepts 0 to 2 arguments (the "N to M" message shape).
#[test]
fn test_error_easter_date_too_many_args() {
    expect_error(
        "<?php easter_date(1, 2, 3);",
        "easter_date() takes 0 to 2 arguments",
    );
}

/// `date_sunrise()` accepts 1 to 6 arguments; a zero-arg call reports the wide "N to M" range.
#[test]
fn test_error_date_sunrise_too_few_args() {
    expect_error(
        "<?php date_sunrise();",
        "date_sunrise() takes 1 to 6 arguments",
    );
}

/// `timezone_version_get()` takes no arguments (the "exactly 0" message shape).
#[test]
fn test_error_timezone_version_get_wrong_args() {
    expect_error(
        "<?php timezone_version_get(1);",
        "timezone_version_get() takes exactly 0 arguments",
    );
}

// -- JSON error tests --

/// Verifies that `json_encode()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_json_encode_no_args() {
    expect_error(
        "<?php json_encode();",
        "json_encode() takes 1 to 3 arguments",
    );
}

/// Verifies that `json_decode()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_json_decode_no_args() {
    expect_error(
        "<?php json_decode();",
        "json_decode() takes 1 to 4 arguments",
    );
}

/// Verifies that `json_validate()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_json_validate_no_args() {
    expect_error(
        "<?php json_validate();",
        "json_validate() takes 1 to 3 arguments",
    );
}

/// Verifies that `json_last_error()` with arguments yields a no-args diagnostic.
#[test]
fn test_error_json_last_error_with_args() {
    expect_error(
        "<?php json_last_error(1);",
        "json_last_error() takes no arguments",
    );
}

/// Verifies that `json_last_error_msg()` with arguments yields a no-args diagnostic.
#[test]
fn test_error_json_last_error_msg_with_args() {
    expect_error(
        "<?php json_last_error_msg(1);",
        "json_last_error_msg() takes no arguments",
    );
}

// -- Regex error tests --

/// Verifies that `preg_match()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_preg_match_no_args() {
    expect_error(
        "<?php preg_match();",
        "preg_match() takes 2 to 5 arguments",
    );
}

/// Verifies that `preg_match()` with only the pattern argument yields a wrong-args diagnostic.
#[test]
fn test_error_preg_match_one_arg() {
    expect_error(
        r#"<?php preg_match("/test/");"#,
        "preg_match() takes 2 to 5 arguments",
    );
}

/// Verifies that `preg_match()` rejects non-variable output arguments for `$matches`.
#[test]
fn test_error_preg_match_matches_must_be_variable() {
    expect_error(
        r#"<?php preg_match("/test/", "test", []);"#,
        "preg_match() parameter $matches must be passed a variable",
    );
}

/// Verifies that `preg_match()` rejects more than the five supported arguments.
#[test]
fn test_error_preg_match_too_many_args() {
    expect_error(
        r#"<?php preg_match("/test/", "test", $matches, 0, 0, 0);"#,
        "preg_match() takes 2 to 5 arguments",
    );
}

/// Verifies that `preg_match_all()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_preg_match_all_no_args() {
    expect_error(
        "<?php preg_match_all();",
        "preg_match_all() takes 2 to 5 arguments",
    );
}

/// Verifies that `preg_replace()` with only two arguments yields a wrong-args diagnostic.
#[test]
fn test_error_preg_replace_wrong_args() {
    expect_error(
        r#"<?php preg_replace("/a/", "b");"#,
        "preg_replace() takes 3 to 5 arguments",
    );
}

/// Verifies a by-reference out-parameter is defined AFTER the call, not before: reading `$matches`
/// before the `preg_match()` call still reports "Undefined variable" (the call writes `$matches`
/// during execution, so a prior read is a genuine undefined-variable use in PHP).
#[test]
fn test_error_preg_match_read_before_call_still_undefined() {
    expect_error(
        r#"<?php
echo $matches;
preg_match('/a/', 'cat', $matches);
"#,
        "Undefined variable: $matches",
    );
}

/// Verifies a by-VALUE builtin argument does NOT define the caller's variable: `strlen($x)` reads
/// `$x` but does not write it, so a later read of an otherwise-undefined `$x` still reports
/// "Undefined variable". Guards against over-marking from the by-reference definite-assignment fix.
#[test]
fn test_error_by_value_builtin_arg_does_not_define() {
    expect_error(
        r#"<?php
strlen($x);
echo $x;
"#,
        "Undefined variable: $x",
    );
}

/// Verifies that `preg_replace()` rejects more than the five supported arguments.
#[test]
fn test_error_preg_replace_too_many_args() {
    expect_error(
        r#"<?php preg_replace("/a/", "b", "c", -1, $count, 0);"#,
        "preg_replace() takes 3 to 5 arguments",
    );
}

/// Verifies that `preg_replace()` rejects a non-variable `$count` output argument.
#[test]
fn test_error_preg_replace_count_must_be_variable() {
    expect_error(
        r#"<?php preg_replace("/a/", "b", "c", -1, 5);"#,
        "preg_replace() parameter $count must be passed a variable",
    );
}

/// Verifies that `preg_replace_callback()` with only two arguments yields a wrong-args
/// diagnostic (PHP allows 3–6).
#[test]
fn test_error_preg_replace_callback_wrong_args() {
    expect_error(
        r#"<?php preg_replace_callback("/a/", function($matches) { return $matches[0]; });"#,
        "preg_replace_callback() takes 3 to 6 arguments",
    );
}

/// Verifies that `preg_replace_callback()` rejects seven arguments (PHP allows 3–6).
#[test]
fn test_error_preg_replace_callback_too_many_args() {
    expect_error(
        r#"<?php preg_replace_callback("/a/", function($m) { return $m[0]; }, "s", 1, 2, 3, 4);"#,
        "preg_replace_callback() takes 3 to 6 arguments",
    );
}

/// Verifies that `preg_split()` with no arguments yields a wrong-args diagnostic.
#[test]
fn test_error_preg_split_no_args() {
    expect_error(
        "<?php preg_split();",
        "preg_split() takes between 2 and 4 arguments",
    );
}

expect_builtin_arity_error!(
    test_error_ini_set_one_arg,
    "<?php ini_set(\"x\");",
    "ini_set() takes exactly 2 arguments"
);

expect_builtin_arity_error!(
    test_error_ini_get_no_args,
    "<?php ini_get();",
    "ini_get() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_get_cfg_var_two_args,
    "<?php get_cfg_var(\"a\", \"b\");",
    "get_cfg_var() takes exactly 1 argument"
);

// -- Hex literal errors --

/// Verifies that concatenating an undefined constant with a string path inside `require` produces a
/// diagnostic that references the undefined constant name.
#[test]
fn test_include_path_with_undefined_const_errors() {
    let err = resolver_error("<?php require UNDEFINED . '/x.php';");
    assert!(
        err.message.contains("UNDEFINED"),
        "message should reference the undefined constant: {}",
        err.message
    );
}

// -- Pre-checker curated-extension `function_exists` fold: correctness gates --
//
// `crate::optimize::function_existence::FunctionExistenceSet::for_pre_check` false-folds
// `function_exists`/`extension_loaded` ONLY for a small curated allowlist of PHP extensions
// elephc never provides (see `NEVER_AVAILABLE_FUNCTION_PREFIXES` in
// `src/optimize/function_existence.rs`). These tests pin the two correctness gates: a call that is
// NOT behind a provably-false guard still errors loudly (an unguarded/always-reached call to an
// extension function elephc cannot resolve), and a plausible-but-uncurated name is left alone by
// the pre-checker fold, so its guard is resolved normally instead of assumed absent.
//
// NOTE: a SEPARATE, narrower, EXACT-name curated allowlist (`crate::types::checker::builtins::
// late_bound`, `apcu_exists`, `opcache_invalidate`, `igbinary_serialize`, ...) makes an UNGUARDED
// call to one of ITS names compile successfully instead — PHP is late-bound, so the compiler
// accepts the call site and lowers it to a catchable `\Error` throw with PHP's exact message
// (see `tests/codegen/late_bound_functions.rs` for that behavior's coverage). That allowlist is
// deliberately much narrower than `NEVER_AVAILABLE_FUNCTION_PREFIXES`'s broad prefix match (no
// prefix wildcards: `apcu_ftch`/a same-family sibling not on the exact list stays loud), so the
// tests below intentionally use names that are prefix-matched for the `function_exists` fold but
// NOT on the late-bound exact allowlist, to keep pinning the "still a compile error" gate.

/// Verifies an UNGUARDED call to a curated never-available extension function still errors loudly:
/// the pre-checker fold only prunes DEAD branches behind a provably-false guard, never the call
/// itself when it is always reached.
#[test]
fn test_error_fastcgi_finish_request_unguarded_call_still_loud() {
    expect_error(
        "<?php fastcgi_finish_request();",
        "Undefined function: fastcgi_finish_request",
    );
}

/// Verifies an UNGUARDED call to an `igbinary_*`-family name that is NOT on the late-bound exact
/// allowlist (`igbinary_serialize`/`igbinary_unserialize` are; this one deliberately is not)
/// still errors loudly, mirroring `test_error_fastcgi_finish_request_unguarded_call_still_loud`.
/// Pins that `NEVER_AVAILABLE_FUNCTION_PREFIXES`'s broad `igbinary_` prefix match (used only for
/// the `function_exists`/`extension_loaded` fold) never leaks into late-bound-call eligibility.
#[test]
fn test_error_igbinary_get_flags_unguarded_call_still_loud() {
    expect_error(
        "<?php igbinary_get_flags();",
        "Undefined function: igbinary_get_flags",
    );
}

/// Verifies an UNGUARDED call to the curated LATE-BOUND `igbinary_serialize` no longer errors at
/// compile time (JURY ADDENDUM: it lowers to a catchable `\Error` throw instead — see
/// `tests/codegen/late_bound_functions.rs` for the runtime-throw coverage). This is the direct
/// regression pin for the pre-L1 expectation this exact call used to error loudly.
#[test]
fn test_igbinary_serialize_unguarded_call_no_longer_compile_errors() {
    expect_ok("<?php igbinary_serialize([1]); echo 'ok';");
}

/// Verifies a TYPO of a curated late-bound name (`apcu_exists` → `apcu_exsts`) still errors
/// loudly at compile time — the late-bound carve-out is EXACT-name-only, never a prefix or
/// fuzzy match (jury addendum #1).
#[test]
fn test_error_late_bound_name_typo_still_loud() {
    expect_error(
        "<?php apcu_exsts('key');",
        "Undefined function: apcu_exsts",
    );
}

/// Verifies a genuinely-undefined USER function with a curated-looking name segment
/// (`apcu_exists_wrapper`) still errors loudly — the late-bound carve-out matches complete
/// names only, never a substring/prefix of a curated name.
#[test]
fn test_error_late_bound_name_substring_still_loud() {
    expect_error(
        "<?php apcu_exists_wrapper('key');",
        "Undefined function: apcu_exists_wrapper",
    );
}

/// Verifies a curated late-bound name used in a top-level `const` initializer still errors
/// loudly: PHP itself rejects ANY function call in a constant expression
/// ("Constant expression contains invalid operations"), and this context is genuinely
/// compile-time-evaluated in elephc (`Checker::compile_time_const_depth`), so the late-bound
/// carve-out must not apply there.
#[test]
fn test_error_late_bound_name_in_const_decl_still_loud() {
    expect_error(
        "<?php const X = apcu_exists('key'); echo X;",
        "Undefined function: apcu_exists",
    );
}

/// Verifies a curated late-bound name used in a class constant initializer still errors loudly,
/// mirroring `test_error_late_bound_name_in_const_decl_still_loud` for class/interface constants.
#[test]
fn test_error_late_bound_name_in_class_const_decl_still_loud() {
    expect_error(
        "<?php class C { const X = apcu_exists('key'); } echo C::X;",
        "Undefined function: apcu_exists",
    );
}

// -- output buffering / get_class_methods (K2): kept-loud forms --

// NOTE: `ob_start()` callback and chunk_size forms are fully supported post-merge
// (see `tests/codegen/io/output_buffering.rs` for the behavioral coverage), so the
// former rejected-form tests were retired.

// `get_class_methods()` requires a literal class-name string or an object of
// statically-known type; a non-literal string is unsupported (never a silent guess).
expect_builtin_arity_error!(
    test_error_get_class_methods_non_literal_string,
    "<?php class C {} function f(string $name) { return get_class_methods($name); } f('C');",
    "get_class_methods() requires a literal class-name string or an object of statically-known type in AOT mode"
);

// `get_class_methods()` on a genuinely dynamic (Mixed) receiver is unsupported.
expect_builtin_arity_error!(
    test_error_get_class_methods_mixed_receiver,
    "<?php function f($x) { return get_class_methods($x); }",
    "get_class_methods() requires a literal class-name string or an object of statically-known type in AOT mode"
);

/// Verifies the pre-checker fold does not over-reach beyond its curated allowlist: a plausible but
/// unknown/absent function name is left unfolded (neither true- nor false-folded) pre-checker, so
/// a call to it inside a NON-provably-false guard is still checked and still errors — the guard's
/// dynamic-looking condition is not assumed false just because the callee is unresolved.
#[test]
fn test_error_uncurated_unknown_function_guard_stays_unfolded_and_errors() {
    expect_error(
        "<?php if (function_exists('totally_made_up_fn_xyz') || true) { totally_made_up_fn_xyz(); }",
        "Undefined function: totally_made_up_fn_xyz",
    );
}

/// serialize() requires exactly one argument.
#[test]
fn test_error_serialize_wrong_args() {
    expect_error("<?php serialize();", "serialize() takes exactly 1 argument");
}

/// unserialize() accepts one or two arguments.
#[test]
fn test_error_unserialize_wrong_args() {
    expect_error("<?php unserialize();", "unserialize() takes 1 or 2 arguments");
}

/// unserialize()'s data argument must be string-compatible.
#[test]
fn test_error_unserialize_non_string_data() {
    expect_error(
        "<?php unserialize([1, 2]);",
        "unserialize() data argument must be string-compatible",
    );
}
