//! Purpose:
//! Assembles the complete mysqli prelude source from its PHP fragments
//! (constants, exception, connection, and — in later tasks — result, statement,
//! multi-query, and procedural aliases), applying `--php-version` gates.
//!
//! Called from:
//! - `crate::mysqli_prelude::parsed_prelude_for_version`.
//!
//! Key details:
//! - Fragments are plain PHP bodies without a `<?php` header; this module owns
//!   the single header, so concatenation order is free (the prelude carries only
//!   hoisted declarations).
//! - Version gates are source rewrites at assembly time, mirroring
//!   `pdo_prelude::prelude_source_for_version`: PHP 8.0 flips the baked
//!   `mysqli_report` default from `ERROR|STRICT` (3) to `OFF` (0).

use crate::php_version::PhpVersion;

/// Returns the complete mysqli prelude source for one PHP compatibility version.
pub(super) fn source_for_version(php_version: PhpVersion) -> String {
    let mut source = String::from("<?php\n");
    source.push_str(super::constants::SRC);
    source.push_str(super::exception::SRC);
    source.push_str(super::connection::SRC);
    source.push_str(super::result::SRC);
    source.push_str(super::statement::SRC);
    source.push_str(super::procedural::SRC);
    if php_version < PhpVersion::Php82 {
        // mysqli::execute_query / mysqli_execute_query are PHP 8.2+.
        remove_version_block(
            &mut source,
            "    // -- elephc PHP >= 8.2 mysqli execute_query begin --",
            "    // -- elephc PHP >= 8.2 mysqli execute_query end --",
        );
        remove_version_block(
            &mut source,
            "// -- elephc PHP >= 8.2 mysqli execute_query begin --",
            "// -- elephc PHP >= 8.2 mysqli execute_query end --",
        );
    }
    if php_version < PhpVersion::Php81 {
        // PHP 8.0's default mysqli_report mode is MYSQLI_REPORT_OFF; 8.1+
        // defaults to MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT.
        source = source.replace(
            "public static int $reportMode = 3;",
            "public static int $reportMode = 0;",
        );
        // mysqli_result::fetch_column and its procedural alias are PHP 8.1+.
        remove_version_block(
            &mut source,
            "    // -- elephc PHP >= 8.1 mysqli fetch_column begin --",
            "    // -- elephc PHP >= 8.1 mysqli fetch_column end --",
        );
        remove_version_block(
            &mut source,
            "// -- elephc PHP >= 8.1 mysqli fetch_column begin --",
            "// -- elephc PHP >= 8.1 mysqli fetch_column end --",
        );
    }
    source
}

/// Removes one inclusive source fragment delimited by stable version-gate
/// comments. Panics when either marker is missing because a renamed prelude
/// marker must fail compiler tests loudly instead of silently exposing a
/// method in the wrong PHP version (same contract as the PDO prelude's helper).
fn remove_version_block(source: &mut String, begin: &str, end: &str) {
    let start = source
        .find(begin)
        .unwrap_or_else(|| panic!("missing mysqli prelude version-gate marker: {begin}"));
    let relative_end = source[start..]
        .find(end)
        .unwrap_or_else(|| panic!("missing mysqli prelude version-gate marker: {end}"));
    let mut finish = start + relative_end + end.len();
    if source.as_bytes().get(finish) == Some(&b'\n') {
        finish += 1;
    }
    source.replace_range(start..finish, "");
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Unit tests for mysqli prelude source assembly: every supported PHP version
    //! tokenizes/parses, and the `mysqli_report` default follows the version gate.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Parsing here mirrors `parsed_prelude_for_version`, so a fragment syntax
    //!   error fails fast in unit tests instead of panicking inside a compile.

    use super::*;

    /// Every supported PHP version's assembled source tokenizes and parses.
    #[test]
    fn every_version_source_tokenizes_and_parses() {
        for version in PhpVersion::ALL {
            let source = source_for_version(version);
            let tokens = crate::lexer::tokenize(&source)
                .unwrap_or_else(|e| panic!("{version:?} prelude must tokenize: {e:?}"));
            crate::parser::parse_internal(&tokens)
                .unwrap_or_else(|e| panic!("{version:?} prelude must parse: {e:?}"));
        }
    }

    /// `execute_query` (method and procedural alias) exists from PHP 8.2 only.
    #[test]
    fn execute_query_is_version_gated() {
        let php81 = source_for_version(PhpVersion::Php81);
        assert!(!php81.contains("execute_query"));
        let php82 = source_for_version(PhpVersion::Php82);
        assert!(php82.contains("public function execute_query"));
        assert!(php82.contains("function mysqli_execute_query"));
    }

    /// `fetch_column` (method and procedural alias) exists from PHP 8.1 only.
    #[test]
    fn fetch_column_is_version_gated() {
        let php80 = source_for_version(PhpVersion::Php80);
        assert!(!php80.contains("fetch_column"));
        let php81 = source_for_version(PhpVersion::Php81);
        assert!(php81.contains("public function fetch_column"));
        assert!(php81.contains("function mysqli_fetch_column"));
    }

    /// PHP 8.0 bakes `mysqli_report` default OFF; 8.1+ bakes ERROR|STRICT.
    #[test]
    fn report_mode_default_follows_php_version() {
        assert!(source_for_version(PhpVersion::Php80)
            .contains("public static int $reportMode = 0;"));
        assert!(source_for_version(PhpVersion::Php81)
            .contains("public static int $reportMode = 3;"));
        assert!(source_for_version(PhpVersion::Php85)
            .contains("public static int $reportMode = 3;"));
    }
}
