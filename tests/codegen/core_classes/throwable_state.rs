//! Purpose:
//! Regressions for PHP-compatible ErrorException constructor state preservation.
//!
//! Called from:
//! - The Core class codegen integration module.
//!
//! Key details:
//! - Native and opaque eval execute the same source against PHP-oracle expectations.
//! - Heap debugging covers retained previous objects and replacement argument temporaries.

use crate::support::*;

const SOURCE: &str = r#"
function coreState(ErrorException $error, Exception $previous): void {
    echo $error->getMessage(), ':', $error->getCode(), ':',
        $error->getPrevious() === $previous ? 'same:' : 'bad:',
        $error->getFile(), ':', $error->getLine(), ':', $error->getSeverity(), ';';
}
function coreStateExtra(): int { echo 'extra;'; return 7; }
$previous = new Exception('old');
$error = new ErrorException('kept', 7, E_WARNING, 'a.php', 3, $previous);
$error->__construct();
coreState($error, $previous);
$error->__construct(filename: null, line: null);
coreState($error, $previous);
$error->__construct(message: 'again', code: 0, previous: null);
coreState($error, $previous);
$replacement = new Exception('new');
$error->__construct(message: 'changed', code: 9, previous: $replacement);
coreState($error, $replacement);
$error->__construct(message: 'alias', previous: $error->getPrevious());
coreState($error, $replacement);
try {
    new ErrorException('too many', 0, E_ERROR, null, null, null, coreStateExtra());
} catch (ArgumentCountError $failure) {
    echo $failure->getMessage();
}
unset($error, $previous, $replacement);
"#;

const EXPECTED: &str = "kept:7:same:a.php:3:1;:7:same:a.php:3:1;again:7:same:a.php:3:1;changed:9:same:a.php:3:1;alias:9:same:a.php:3:1;extra;ErrorException::__construct() expects at most 6 arguments, 7 given";

/// Native constructor calls distinguish omitted messages and preserve zero/null field arguments.
#[test]
fn test_core_review_error_exception_reinitialization_preserves_state() {
    let out = compile_and_run_with_heap_debug(&format!("<?php\n{SOURCE}"));
    assert!(out.success, "stdout: {}\nstderr: {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, EXPECTED);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Opaque eval retains the same defaults, previous identity, and constructor arity contract.
#[test]
fn test_core_review_error_exception_reinitialization_preserves_state_eval() {
    let encoded = SOURCE.replace('\\', "\\\\").replace('\'', "\\'");
    let out = compile_and_run_with_heap_debug(&format!("<?php $source = '{encoded}' . ' // ' . $argc; eval($source);"));
    assert!(out.success, "stdout: {}\nstderr: {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, EXPECTED);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}
