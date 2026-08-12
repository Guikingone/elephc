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
    if php_version < PhpVersion::Php81 {
        // PHP 8.0's default mysqli_report mode is MYSQLI_REPORT_OFF; 8.1+
        // defaults to MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT.
        source = source.replace(
            "public static int $reportMode = 3;",
            "public static int $reportMode = 0;",
        );
    }
    source
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
