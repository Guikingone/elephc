//! Purpose:
//! Defines the source-language mode selected from a physical input path.
//! Centralizes `.lfc` classification so every file loader agrees on tag and strict-mode semantics.
//!
//! Called from:
//! - `crate::pipeline::compile()` for the entry source.
//! - `crate::resolver` and `crate::autoload` for additional physical source files.
//!
//! Key details:
//! - Only `.lfc` opts into tagless elephc source; every other path preserves tagged-PHP behavior.
//! - Classification is ASCII case-insensitive and never changes output-path naming.

use std::cell::Cell;
use std::collections::HashSet;
use std::path::Path;

use crate::errors::CompileError;
use crate::parser::ast::Program;

/// Language profile selected for one physical source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceMode {
    /// Tagged PHP-compatible source that requires the normal `<?php` opening tag.
    Php,
    /// Tagless elephc source with every elephc extension available.
    Lfc,
    /// Compiler-generated source that is never subject to the user strict-PHP audit.
    Internal,
}

thread_local! {
    /// Source mode inherited by AST nodes created during one parser invocation.
    static CURRENT_PARSE_MODE: Cell<SourceMode> = const { Cell::new(SourceMode::Internal) };
}

impl SourceMode {
    /// Classifies a physical path, treating only a case-insensitive `.lfc` suffix as tagless.
    pub fn from_path(path: &Path) -> Self {
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("lfc"))
        {
            Self::Lfc
        } else {
            Self::Php
        }
    }

    /// Returns whether this source mode requires a user-written `<?php` opening tag.
    pub fn requires_open_tag(self) -> bool {
        matches!(self, Self::Php)
    }

    /// Returns whether an invocation-level strict-PHP request applies to this source.
    pub fn strict_php_is_effective(self, requested: bool) -> bool {
        requested && matches!(self, Self::Php)
    }
}

/// Returns whether Composer discovery should inspect a path as PHP/LFC source.
///
/// Physical includes remain PHP-compatible regardless of suffix, but Composer's
/// directory walkers intentionally discover only files named `.php` or `.lfc`.
pub fn is_composer_source_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("php") || extension.eq_ignore_ascii_case("lfc")
        })
}

/// Removes the recognized source suffix from one Composer-relative path component.
pub fn composer_source_stem(component: &str) -> String {
    Path::new(component)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(component)
        .to_string()
}

/// RAII guard restoring the parser's previous source mode on drop.
pub(crate) struct ParseModeGuard {
    previous: SourceMode,
}

impl Drop for ParseModeGuard {
    /// Restores the parser source mode active before the nested parse.
    fn drop(&mut self) {
        CURRENT_PARSE_MODE.with(|cell| cell.set(self.previous));
    }
}

/// Runs `f` while parser-created AST nodes inherit `mode`.
pub(crate) fn with_parse_mode<T>(mode: SourceMode, f: impl FnOnce() -> T) -> T {
    let _guard = scoped_parse_mode(mode);
    f()
}

/// Installs one parser/source reconstruction mode until the returned guard is dropped.
pub(crate) fn scoped_parse_mode(mode: SourceMode) -> ParseModeGuard {
    let previous = CURRENT_PARSE_MODE.with(|cell| cell.replace(mode));
    ParseModeGuard { previous }
}

/// Returns the source mode assigned to AST nodes created at the current parse site.
pub(crate) fn current_parse_mode() -> SourceMode {
    CURRENT_PARSE_MODE.with(Cell::get)
}

/// Applies path-dependent post-parse processing shared by every physical source loader.
///
/// Magic constants retain the real path, strict PHP audits the unfiltered physical
/// program, and conditional compilation runs last so inactive extension syntax in a
/// PHP file cannot evade the audit.
pub fn finalize_physical_program(
    program: Program,
    path: &Path,
    mode: SourceMode,
    defines: &HashSet<String>,
) -> Result<Program, CompileError> {
    let program = crate::magic_constants::substitute_file_and_scope_constants(program, path);
    crate::strict_php::check_file_with_mode(&program, &path.display().to_string(), mode)?;
    Ok(crate::conditional::apply(program, defines))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies `.lfc` classification is case-insensitive while other suffixes remain PHP mode.
    #[test]
    fn classifies_only_lfc_as_tagless() {
        assert_eq!(SourceMode::from_path(Path::new("main.lfc")), SourceMode::Lfc);
        assert_eq!(SourceMode::from_path(Path::new("main.LFC")), SourceMode::Lfc);
        assert_eq!(SourceMode::from_path(Path::new("main.php")), SourceMode::Php);
        assert_eq!(SourceMode::from_path(Path::new("bootstrap.inc")), SourceMode::Php);
        assert_eq!(SourceMode::from_path(Path::new("script")), SourceMode::Php);
    }

    /// Verifies Composer discovery accepts only the two physical source suffixes.
    #[test]
    fn classifies_composer_source_paths() {
        assert!(is_composer_source_path(Path::new("src/App.php")));
        assert!(is_composer_source_path(Path::new("src/App.LFC")));
        assert!(!is_composer_source_path(Path::new("src/App.inc")));
        assert_eq!(composer_source_stem("App.lfc"), "App");
    }

    /// Verifies strict PHP applies only to PHP-mode user source.
    #[test]
    fn strict_php_is_source_mode_aware() {
        assert!(SourceMode::Php.strict_php_is_effective(true));
        assert!(!SourceMode::Php.strict_php_is_effective(false));
        assert!(!SourceMode::Lfc.strict_php_is_effective(true));
        assert!(!SourceMode::Internal.strict_php_is_effective(true));
    }
}
