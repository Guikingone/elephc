//! Purpose:
//! Reads and edits the `[php-ext]` section of a project manifest, which declares
//! which real PHP extensions a program hosts.
//!
//! Called from:
//! - `elephc php-ext add/remove/list` and extension resolution during compilation.
//!
//! Key details:
//! - Mirrors `native_deps::manifest`: strict about its own section, and blind to
//!   every other one, so `[native]` and hand-written TOML survive an edit intact.
//! - Versions are exact. A range would make a build's contents depend on when it
//!   ran, and an extension's ABI surface is exactly what must not drift.
//! - Declaring an extension is not the same as admitting it: `php_ext::admission`
//!   still has to see the built symbols. The manifest records intent only.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::str::FromStr;

use toml_edit::{value, DocumentMut, Item, Table};

use crate::native_deps::{NativeError, NativeErrorKind};

fn manifest_error(message: impl Into<String>) -> NativeError {
    NativeError::new(NativeErrorKind::Manifest, message)
}

/// Comment-preserving manifest document and validated extension selection.
#[derive(Clone, Debug)]
pub struct PhpExtManifest {
    document: DocumentMut,
    extensions: BTreeMap<String, String>,
}

impl PhpExtManifest {
    /// Creates a minimal schema-1 `[php-ext]` manifest in memory.
    pub fn new() -> Self {
        let mut document = DocumentMut::new();
        document["php-ext"] = Item::Table(Table::new());
        document["php-ext"]["schema"] = value(1);
        document["php-ext"]["extensions"] = Item::Table(Table::new());
        Self {
            document,
            extensions: BTreeMap::new(),
        }
    }

    /// Parses and strictly validates the `[php-ext]` section.
    ///
    /// A manifest with no `[php-ext]` section is valid and simply declares no
    /// extensions — hosting is opt-in, so its absence is not an error.
    pub fn parse(text: &str) -> Result<Self, NativeError> {
        let document = DocumentMut::from_str(text)
            .map_err(|error| manifest_error(format!("invalid TOML: {error}")))?;

        let Some(section) = document.get("php-ext").and_then(Item::as_table) else {
            return Ok(Self {
                document,
                extensions: BTreeMap::new(),
            });
        };

        for (key, _) in section.iter() {
            if !matches!(key, "schema" | "extensions") {
                return Err(manifest_error(format!("unknown key 'php-ext.{key}'")));
            }
        }
        if section.get("schema").and_then(Item::as_integer) != Some(1) {
            return Err(manifest_error(
                "php-ext.schema is required and must equal 1",
            ));
        }
        let table = section
            .get("extensions")
            .and_then(Item::as_table)
            .ok_or_else(|| manifest_error("missing [php-ext.extensions] table"))?;

        let mut extensions = BTreeMap::new();
        let mut folded = BTreeSet::new();
        for (name, item) in table.iter() {
            validate_extension_name(name)?;
            if !folded.insert(name.to_ascii_lowercase()) {
                return Err(manifest_error(format!(
                    "duplicate case-variant extension '{name}'"
                )));
            }
            let version = item.as_str().ok_or_else(|| {
                manifest_error(format!(
                    "php extension '{name}' must be an exact-version string"
                ))
            })?;
            validate_exact_version(name, version)?;
            extensions.insert(name.to_string(), version.to_string());
        }

        Ok(Self {
            document,
            extensions,
        })
    }

    /// Reads and parses a manifest from disk.
    pub fn load(path: &Path) -> Result<Self, NativeError> {
        let text = fs::read_to_string(path)
            .map_err(|error| NativeError::io("read php-ext manifest", path, error))?;
        Self::parse(&text).map_err(|error| error.with_path(path))
    }

    /// Declared extensions in deterministic name order.
    pub fn extensions(&self) -> &BTreeMap<String, String> {
        &self.extensions
    }

    /// Adds or replaces an extension, creating the section when absent.
    pub fn set_extension(&mut self, name: &str, version: &str) -> Result<(), NativeError> {
        validate_extension_name(name)?;
        validate_exact_version(name, version)?;
        if self.document.get("php-ext").and_then(Item::as_table).is_none() {
            self.document["php-ext"] = Item::Table(Table::new());
            self.document["php-ext"]["schema"] = value(1);
            self.document["php-ext"]["extensions"] = Item::Table(Table::new());
        }
        self.document["php-ext"]["extensions"][name] = value(version);
        self.extensions.insert(name.to_string(), version.to_string());
        Ok(())
    }

    /// Removes an extension; returns whether it was declared.
    pub fn remove_extension(&mut self, name: &str) -> bool {
        let removed = self.extensions.remove(name).is_some();
        if removed {
            if let Some(table) = self
                .document
                .get_mut("php-ext")
                .and_then(Item::as_table_mut)
                .and_then(|section| section.get_mut("extensions"))
                .and_then(Item::as_table_mut)
            {
                table.remove(name);
            }
        }
        removed
    }

    /// Renders the manifest, preserving unrelated sections and comments.
    pub fn render(&self) -> String {
        self.document.to_string()
    }
}

impl Default for PhpExtManifest {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension names follow PECL: lowercase ASCII, digits, underscore.
fn validate_extension_name(name: &str) -> Result<(), NativeError> {
    if name.is_empty() {
        return Err(manifest_error("extension name must not be empty"));
    }
    if !name
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        return Err(manifest_error(format!(
            "extension key '{name}' must be lowercase ASCII letters, digits or underscore"
        )));
    }
    Ok(())
}

/// Exact versions only: `5.1.28`, never `^5.1` or `*`. A build's contents must
/// not depend on when it ran, least of all for an ABI surface.
fn validate_exact_version(name: &str, version: &str) -> Result<(), NativeError> {
    if version.is_empty() {
        return Err(manifest_error(format!(
            "php extension '{name}' has an empty version"
        )));
    }
    // Ranges are also caught by the character check below, but only this branch
    // can say *why* — "not a range" is actionable, "unexpected characters" is not.
    let looks_like_range = version
        .starts_with(['^', '~', '>', '<', '=', '*'])
        || version.contains(['*', ' ', ',', '|']);
    if looks_like_range {
        return Err(manifest_error(format!(
            "php extension '{name}' version '{version}' must be exact, not a range"
        )));
    }
    if !version
        .bytes()
        .all(|b| b.is_ascii_digit() || b == b'.' || b == b'-' || b.is_ascii_alphabetic())
    {
        return Err(manifest_error(format!(
            "php extension '{name}' version '{version}' has unexpected characters"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_declared_extensions() {
        let text = r#"
[php-ext]
schema = 1

[php-ext.extensions]
simdjson = "4.0.0"
apcu = "5.1.28"
"#;
        let manifest = PhpExtManifest::parse(text).expect("valid manifest");
        assert_eq!(manifest.extensions().len(), 2);
        assert_eq!(
            manifest.extensions().get("apcu").map(String::as_str),
            Some("5.1.28")
        );
    }

    /// Hosting is opt-in: a project that declares none is not in error.
    #[test]
    fn a_manifest_without_the_section_declares_nothing() {
        let text = "[native]\nschema = 1\n\n[native.dependencies]\n";
        let manifest = PhpExtManifest::parse(text).expect("valid manifest");
        assert!(manifest.extensions().is_empty());
    }

    /// The whole point of mirroring native_deps: editing one section must not
    /// disturb another, nor drop comments.
    #[test]
    fn editing_preserves_unrelated_sections_and_comments() {
        let text = r#"# project manifest
[native]
schema = 1

[native.dependencies]
zlib = "1.3.1"

[php-ext]
schema = 1

[php-ext.extensions]
simdjson = "4.0.0"
"#;
        let mut manifest = PhpExtManifest::parse(text).expect("valid");
        manifest.set_extension("apcu", "5.1.28").expect("accepted");
        let rendered = manifest.render();
        assert!(rendered.contains("# project manifest"), "comment kept");
        assert!(rendered.contains("[native.dependencies]"), "native kept");
        assert!(rendered.contains("zlib = \"1.3.1\""), "native entry kept");
        assert!(rendered.contains("apcu = \"5.1.28\""), "new entry written");
    }

    #[test]
    fn adds_the_section_when_absent() {
        let mut manifest = PhpExtManifest::parse("[native]\nschema = 1\n\n[native.dependencies]\n")
            .expect("valid");
        manifest.set_extension("simdjson", "4.0.0").expect("ok");
        let rendered = manifest.render();
        assert!(rendered.contains("[php-ext.extensions]"));
        assert!(rendered.contains("[native.dependencies]"), "native intact");
    }

    #[test]
    fn removes_a_declared_extension() {
        let mut manifest = PhpExtManifest::new();
        manifest.set_extension("ds", "1.5.0").expect("ok");
        assert!(manifest.remove_extension("ds"));
        assert!(!manifest.remove_extension("ds"), "second removal is a no-op");
        assert!(manifest.extensions().is_empty());
        assert!(!manifest.render().contains("ds ="));
    }

    /// Ranges must be refused *as ranges*. Asserting only that the call fails is
    /// vacuous — the character check refuses these strings anyway, so such a test
    /// stays green even with the range rule deleted (confirmed by sentinel
    /// mutation). Asserting on the message is what actually pins the rule.
    #[test]
    fn rejects_version_ranges_with_an_actionable_message() {
        let mut manifest = PhpExtManifest::new();
        for bad in ["^5.1", "~5.1.0", ">=5.0", "*", "5.1.* ", "5.1 || 5.2"] {
            let error = manifest
                .set_extension("apcu", bad)
                .expect_err("a build's ABI surface cannot depend on when it ran");
            let rendered = error.to_string();
            assert!(
                rendered.contains("must be exact, not a range"),
                "'{bad}' should be reported as a range, got: {rendered}"
            );
        }
        assert!(manifest.set_extension("apcu", "5.1.28").is_ok());
    }

    /// The neighbouring rule: characters that are not range syntax but still do
    /// not belong in a version.
    #[test]
    fn rejects_versions_with_unexpected_characters() {
        let mut manifest = PhpExtManifest::new();
        let error = manifest
            .set_extension("apcu", "5.1.28/beta")
            .expect_err("slash is not a version character");
        assert!(error.to_string().contains("unexpected characters"));
    }

    #[test]
    fn rejects_unknown_keys_in_its_own_section() {
        let text = "[php-ext]\nschema = 1\nnope = true\n\n[php-ext.extensions]\n";
        assert!(PhpExtManifest::parse(text).is_err());
    }

    #[test]
    fn rejects_a_wrong_schema() {
        let text = "[php-ext]\nschema = 2\n\n[php-ext.extensions]\n";
        assert!(PhpExtManifest::parse(text).is_err());
    }

    #[test]
    fn rejects_case_variant_duplicates() {
        // Parsed rather than built, since the setter validates names first.
        let text = "[php-ext]\nschema = 1\n\n[php-ext.extensions]\nsimdjson = \"4.0.0\"\n";
        assert!(PhpExtManifest::parse(text).is_ok());
        let mut manifest = PhpExtManifest::new();
        assert!(
            manifest.set_extension("SimdJson", "4.0.0").is_err(),
            "uppercase names are refused outright"
        );
    }

    #[test]
    fn rejects_non_string_versions() {
        let text = "[php-ext]\nschema = 1\n\n[php-ext.extensions]\napcu = 5\n";
        assert!(PhpExtManifest::parse(text).is_err());
    }
}
