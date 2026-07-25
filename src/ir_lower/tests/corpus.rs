//! Purpose:
//! Corpus validation tests for AST-to-EIR lowering over real example programs.
//!
//! Called from:
//! - `crate::ir_lower::tests`.
//!
//! Key details:
//! - Exercises the full frontend ordering, including resolver and autoload, on
//!   each `examples/*/main.php` fixture before EIR validation.

use std::path::{Path, PathBuf};

/// Verifies every checked example program lowers to validated printable EIR.
///
/// The `strict-php` example is lowered with strict-PHP mode enabled, matching
/// its documented `elephc --strict-php` invocation: it deliberately declares a
/// user function named after an extension builtin, which only PHP-compatible
/// (strict) resolution accepts.
#[test]
fn lowers_examples_corpus() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut fixtures = example_main_files(root);
    fixtures.sort();
    assert!(!fixtures.is_empty(), "expected example PHP fixtures");

    for fixture in fixtures {
        let strict = fixture
            .parent()
            .and_then(|dir| dir.file_name())
            .is_some_and(|name| name == "strict-php");
        // RAII guard: if lowering a strict fixture panics, the guard still
        // restores the state during unwinding, so no later fixture can
        // accidentally run with strict mode inherited.
        let _guard = strict.then(crate::strict_php::scoped_enable);
        let module = super::lower_file(&fixture);
        assert!(
            !module.functions.is_empty(),
            "expected at least main function for {}",
            fixture.display()
        );
    }
}

/// Returns all example `main.php` fixtures in deterministic order, excluding
/// examples that only type-check once a feature prelude has been injected.
///
/// The corpus lowers each fixture in plain (CLI) mode, which does not inject the
/// feature preludes the pipeline adds during a real compile — the `--web` request
/// prelude (`src/web_prelude.rs`) or the pay-for-use OPcache prelude
/// (`src/opcache_prelude.rs`). Examples that rely on such a prelude — session
/// functions, request superglobals, or the prelude-provided `opcache_*` functions —
/// reference symbols that do not exist in plain CLI-mode lowering and legitimately
/// fail type checking here, so they are skipped rather than treated as failures.
fn example_main_files(root: &Path) -> Vec<PathBuf> {
    let examples = root.join("examples");
    std::fs::read_dir(&examples)
        .expect("examples directory should exist")
        .map(|entry| entry.expect("example entry").path().join("main.php"))
        .filter(|path| path.exists())
        .filter(|path| !example_requires_prelude(path))
        .collect()
}

/// Returns true when an example directory only compiles once the pipeline injects a
/// feature prelude (the `--web` request prelude or the OPcache prelude) and must be
/// skipped by the plain CLI-mode corpus lowering test.
fn example_requires_prelude(main_php: &Path) -> bool {
    const PRELUDE_ONLY_EXAMPLES: &[&str] = &[
        "web-session",
        "web-session-trans-sid",
        "web-session-upload",
        // OPcache introspection functions are provided by the pay-for-use OPcache
        // prelude, which the plain CLI-mode corpus lowering does not inject.
        "opcache_get_configuration",
    ];
    main_php
        .parent()
        .and_then(|dir| dir.file_name())
        .and_then(|name| name.to_str())
        .is_some_and(|name| PRELUDE_ONLY_EXAMPLES.contains(&name))
}
