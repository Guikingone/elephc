//! Purpose:
//! The mysqli standard-library surface (MySQL / MariaDB), implemented in
//! elephc-PHP as a second front-end over the existing `elephc_pdo` bridge.
//! Declares `mysqli`, `mysqli_stmt`, `mysqli_result`, `mysqli_sql_exception`,
//! the `MYSQLI_*` constants, and the `mysqli_*` procedural aliases — never the
//! PDO classes: a mysqli-only program must not see `PDO` / `PDOStatement` or
//! throw `PDOException`.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via
//!   `inject_if_used`, after the PDO prelude injection and before name
//!   resolution.
//!
//! Key details:
//! - The prelude is injected only when the program references a mysqli symbol
//!   (see `detect`) or `--with-mysqli` forces it; injection also prepends the
//!   shared `extern "elephc_pdo"` block via
//!   `pdo_prelude::inject_bridge_externs`, which is idempotent so a program
//!   using both PDO and mysqli declares the externs exactly once.
//! - Declaring the shared externs is what makes the checker record the
//!   `elephc_pdo` staticlib, so mysqli programs link the bridge with no new
//!   `BRIDGES` entry.
//! - `extension_loaded('mysqli')` reporting is surface-based: the pipeline
//!   records the "mysqli" surface when this prelude is injected (the shared
//!   archive alone identifies no extension). `mysqlnd` is never reported.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::parser::ast::{Program, Stmt};
use crate::php_version::PhpVersion;

mod connection;
mod constants;
mod detect;
mod exception;
mod fragments;
mod procedural;

static PARSED_PRELUDE_CACHE: OnceLock<Mutex<HashMap<PhpVersion, Program>>> = OnceLock::new();

/// Returns whether the program references the mysqli surface. Exposed so the
/// pipeline (and the test harnesses) can record the "mysqli" PHP surface for
/// `extension_loaded()` reporting using the same detection that decides
/// prelude injection.
pub fn program_uses_mysqli(program: &[Stmt]) -> bool {
    detect::program_uses_mysqli(program)
}

/// Prepends the mysqli prelude (and, idempotently, the shared `elephc_pdo`
/// extern block) to `program` when it references the mysqli surface, so the
/// classes and externs compile through the normal pipeline only for
/// mysqli-using programs. The prelude carries only declarations, which are
/// hoisted, so prepending does not change top-level execution order.
///
/// `force` (set by `--with-mysqli`) bypasses the usage scan so the mysqli
/// surface is always injected and the bridge always linked. The PDO classes are
/// never injected by this path.
pub fn inject_if_used(program: Program, force: bool, php_version: PhpVersion) -> Program {
    if !force && !detect::program_uses_mysqli(&program) {
        return program;
    }
    let mut combined = parsed_prelude_for_version(php_version);
    combined.extend(program);
    // Shared with the PDO prelude; idempotent, so whichever surface injects
    // second finds the block already declared and leaves it alone.
    crate::pdo_prelude::inject_bridge_externs(combined)
}

/// Returns the mysqli prelude PHP fragments as `(label, source)` pairs, for
/// source-level scans (the prelude parity gates).
#[allow(dead_code)] // consumed by the cfg(test) parity gate; unused in the bin target
pub fn fragment_sources() -> &'static [(&'static str, &'static str)] {
    &[
        ("mysqli_prelude(constants)", constants::SRC),
        ("mysqli_prelude(exception)", exception::SRC),
        ("mysqli_prelude(connection)", connection::SRC),
        ("mysqli_prelude(procedural)", procedural::SRC),
    ]
}

/// Returns an independent clone of the parsed mysqli prelude for one PHP
/// version. The compiler mutates every injected AST in later passes, so the
/// cache stores an immutable template and clones it per compilation.
fn parsed_prelude_for_version(php_version: PhpVersion) -> Program {
    let cache = PARSED_PRELUDE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cache
        .entry(php_version)
        .or_insert_with(|| {
            let source = fragments::source_for_version(php_version);
            let tokens =
                crate::lexer::tokenize(&source).expect("mysqli prelude must tokenize");
            crate::parser::parse_internal(&tokens).expect("mysqli prelude must parse")
        })
        .clone()
}
