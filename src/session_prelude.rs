//! The PHP CLI session extension surface, backed by the same session implementation as `--web`.
//!
//! Called from `crate::pipeline::compile` after include/autoload expansion so the CLI program
//! and its dynamically loaded declarations see the standard `session_*` functions.
//!
//! Key details:
//! - This is the PHP session implementation built by `web_prelude::build`, not a synthetic
//!   success stub. Both SAPIs share the PHP-level implementation and state/file-I/O bridge.
//! - Only session declarations are injected here; HTTP request initialization and the web entry
//!   point remain exclusive to `--web`.

use std::path::Path;

use crate::optimize::reachability::PreludeInventory;
use crate::parser::ast::Program;
use crate::web_prelude::PhpVersion;

/// Reachability group retained because the PHP session extension is present in the CLI profile,
/// even when session calls are made only from source the compiler loads dynamically.
pub(crate) const SESSION_PRELUDE_GROUP: &str = "session";

/// Injects the standard session surface into the CLI profile; `--web` already owns it.
pub(crate) fn inject_cli_session_surface(
    program: Program,
    web: bool,
    php_version: PhpVersion,
    ini_overrides: &[(String, String)],
    entry_path: &Path,
    inventory: &mut PreludeInventory,
) -> Program {
    if web {
        return program;
    }

    let declarations = crate::web_prelude::build::session_declarations(php_version, ini_overrides);
    let declarations = crate::magic_constants::substitute_file_constants(declarations, entry_path);
    inventory.record_program(SESSION_PRELUDE_GROUP, &declarations);

    let mut combined = declarations;
    combined.extend(program);
    combined
}
