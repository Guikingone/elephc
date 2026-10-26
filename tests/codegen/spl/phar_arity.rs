//! Purpose:
//! Runtime tests for the internal-class argument-count error on the Phar class family.
//!
//! Called from:
//! - `cargo test --test codegen_tests` through the SPL test module.
//!
//! Key details:
//! - php defers an internal class's arity error to a catchable `ArgumentCountError` at run
//!   time; these tests pin php's message wording for Phar's method signatures.

use crate::support::*;

/// Verifies that `Phar::addFromString()`'s missing second argument raises php's catchable
/// `ArgumentCountError` with php's wording.
#[test]
fn test_phar_add_from_string_arity_is_a_runtime_argument_count_error() {
    let out = compile_and_run(
        r#"<?php
$p = new Phar("x.phar");
try {
    $p->addFromString("entry.txt");
} catch (ArgumentCountError $e) {
    echo $e->getMessage(), "\n";
}
echo "done\n";
"#,
    );
    assert_eq!(
        out,
        concat!(
            "Phar::addFromString() expects exactly 2 arguments, 1 given\n",
            "done\n",
        )
    );
}

/// Verifies that `Phar::compressFiles()` without its compression argument raises php's
/// catchable `ArgumentCountError`.
#[test]
fn test_phar_compress_files_arity_is_a_runtime_argument_count_error() {
    let out = compile_and_run(
        r#"<?php
$p = new Phar("x.phar");
try {
    $p->compressFiles();
} catch (ArgumentCountError $e) {
    echo $e->getMessage(), "\n";
}
echo "done\n";
"#,
    );
    assert_eq!(
        out,
        concat!(
            "Phar::compressFiles() expects exactly 1 argument, 0 given\n",
            "done\n",
        )
    );
}

/// Verifies that `Phar::setMetadata()` without its metadata argument raises php's catchable
/// `ArgumentCountError`.
#[test]
fn test_phar_set_metadata_arity_is_a_runtime_argument_count_error() {
    let out = compile_and_run(
        r#"<?php
$p = new Phar("x.phar");
try {
    $p->setMetadata();
} catch (ArgumentCountError $e) {
    echo $e->getMessage(), "\n";
}
echo "done\n";
"#,
    );
    assert_eq!(
        out,
        concat!(
            "Phar::setMetadata() expects exactly 1 argument, 0 given\n",
            "done\n",
        )
    );
}
