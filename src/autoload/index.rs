//! Purpose:
//! Builds the static namespace and class-file index used by the AOT autoload pass.
//! Reads PSR-4, PSR-0, classmap, eager-file, and exclusion rules from supported manifests.
//!
//! Called from:
//! - `crate::autoload::Registry::build()`
//!
//! Key details:
//! - Produces FQN-to-path mappings and eager source entries for compile-time inclusion.
//! - Production and development mapping sections are merged because compiled binaries have one source graph.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use crate::errors::CompileWarning;
use crate::parser::ast::{Stmt, StmtKind};
use crate::span::Span;

/// Compiled view of every autoload section found in the project.
pub struct AutoloadIndex {
    fqn_to_path: HashMap<String, PathBuf>,
    files_to_include: Vec<PathBuf>,
    /// Classmap files the scan could not read; each one hides every class it declares.
    warnings: Vec<CompileWarning>,
}

impl AutoloadIndex {
    /// Builds the index from the nearest supported root manifest and nested dependency manifests.
    /// Returns an empty index when no supported manifest exists at or above the entry directory.
    pub fn from_project_root(project_root: &Path) -> Self {
        let Some(project_root) = nearest_autoload_manifest_root(project_root) else {
            return Self {
                fqn_to_path: HashMap::new(),
                files_to_include: Vec::new(),
                warnings: Vec::new(),
            };
        };
        let mut builder = IndexBuilder::default();
        for manifest in autoload_manifest_paths(&project_root) {
            builder.load_manifest(&manifest, true);
        }
        for manifest in nested_autoload_manifest_paths(&project_root) {
            builder.load_manifest(&manifest, false);
        }
        AutoloadIndex {
            fqn_to_path: builder.fqn_to_path,
            files_to_include: builder.files_to_include,
            warnings: builder.warnings,
        }
    }

    /// Diagnostics collected while building the index, one per unreadable classmap file.
    pub fn warnings(&self) -> &[CompileWarning] {
        &self.warnings
    }

    /// Look up the file path for a given fully-qualified class name.
    pub fn lookup(&self, fqn: &str) -> Option<&Path> {
        let key = fqn.trim_start_matches('\\');
        self.fqn_to_path.get(key).map(PathBuf::as_path)
    }

    /// True when the index has no PSR-4 mappings and no files entries.
    pub fn is_empty(&self) -> bool {
        self.fqn_to_path.is_empty() && self.files_to_include.is_empty()
    }

    /// Files listed under `autoload.files` / `autoload-dev.files`.
    pub fn files(&self) -> &[PathBuf] {
        &self.files_to_include
    }
}

/// Finds the nearest ancestor containing a structurally recognized autoload manifest.
fn nearest_autoload_manifest_root(entry_dir: &Path) -> Option<PathBuf> {
    entry_dir
        .ancestors()
        .find(|candidate| !autoload_manifest_paths(candidate).is_empty())
        .map(Path::to_path_buf)
}

/// Returns direct JSON manifests containing a supported autoload section.
fn autoload_manifest_paths(dir: &Path) -> Vec<PathBuf> {
    let mut manifests = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path_has_json_extension(path))
        .filter(|path| manifest_has_autoload_section(path))
        .collect::<Vec<_>>();
    manifests.sort();
    manifests
}

/// Recursively discovers structurally recognized manifests below the project root.
fn nested_autoload_manifest_paths(root: &Path) -> Vec<PathBuf> {
    let mut pending = VecDeque::from([root.to_path_buf()]);
    let mut visited = HashSet::new();
    let mut manifests = Vec::new();
    while let Some(dir) = pending.pop_front() {
        let canonical = dir.canonicalize().unwrap_or_else(|_| dir.clone());
        if !visited.insert(canonical) {
            continue;
        }
        let mut entries = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.path());
        for entry in entries {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                pending.push_back(path);
            } else if file_type.is_file()
                && path_has_json_extension(&path)
                && manifest_has_autoload_section(&path)
                && path.parent() != Some(root)
            {
                manifests.push(path);
            }
        }
    }
    manifests
}

/// Returns whether a path uses the JSON extension, case-insensitively.
fn path_has_json_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
}

/// Returns whether a JSON document exposes at least one supported autoload section.
fn manifest_has_autoload_section(path: &Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    if !content.contains("\"autoload\"") && !content.contains("\"autoload-dev\"") {
        return false;
    }
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    ["autoload", "autoload-dev"]
        .into_iter()
        .any(|key| json.get(key).is_some_and(serde_json::Value::is_object))
}

#[derive(Default)]
/// Accumulates static autoload index entries while reading supported manifests.
struct IndexBuilder {
    fqn_to_path: HashMap<String, PathBuf>,
    files_to_include: Vec<PathBuf>,
    warnings: Vec<CompileWarning>,
}

impl IndexBuilder {
    /// Loads one supported manifest, optionally collecting its eager source entries.
    ///
    /// Nested manifests always contribute mappings. Only the root manifest contributes eager
    /// entries directly; nested eager entries remain owned by the loader source the program
    /// actually includes, avoiding duplicate execution and unrelated graph expansion.
    fn load_manifest(&mut self, manifest_path: &Path, include_files: bool) {
        let Some(dir) = manifest_path.parent() else {
            return;
        };
        let Ok(content) = std::fs::read_to_string(manifest_path) else {
            return;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            return;
        };
        // Read both the production and dev autoload sections; the AOT
        // model has no production/test split, so they merge.
        for section_key in ["autoload", "autoload-dev"] {
            if let Some(section) = json.get(section_key) {
                self.load_section(dir, section, include_files);
            }
        }
    }

    /// Parse one autoload section (psr-4, psr-0, classmap, files) and update the index.
    fn load_section(
        &mut self,
        base_dir: &Path,
        section: &serde_json::Value,
        include_files: bool,
    ) {
        let excludes = section
            .get("exclude-from-classmap")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| normalize_exclude_pattern(base_dir, s)))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if let Some(psr4) = section.get("psr-4").and_then(|p| p.as_object()) {
            self.read_psr_namespaced(base_dir, psr4, walk_psr4);
        }
        if let Some(psr0) = section.get("psr-0").and_then(|p| p.as_object()) {
            self.read_psr_namespaced(base_dir, psr0, walk_psr0);
        }
        if let Some(classmap) = section.get("classmap").and_then(|c| c.as_array()) {
            self.read_classmap(base_dir, classmap, &excludes);
        }
        if include_files {
            if let Some(files) = section.get("files").and_then(|f| f.as_array()) {
                self.read_files(base_dir, files);
            }
        }
    }

    /// Shared driver for `psr-4` and `psr-0`. Sorts prefixes by length
    /// descending so longer prefixes claim FQNs before shorter ones (PHP
    /// longest-prefix-wins rule).
    fn read_psr_namespaced(
        &mut self,
        base_dir: &Path,
        prefix_map: &serde_json::Map<String, serde_json::Value>,
        walker: fn(&Path, &str, &Path, &mut HashMap<String, PathBuf>),
    ) {
        let mut prefixes: Vec<&String> = prefix_map.keys().collect();
        prefixes.sort_by_key(|p| std::cmp::Reverse(p.len()));
        for prefix in prefixes {
            for dir in extract_paths(&prefix_map[prefix]) {
                let root = base_dir.join(dir);
                walker(&root, prefix, &root, &mut self.fqn_to_path);
            }
        }
    }

    /// Scan a classmap entry and populate the FQN index.
    fn read_classmap(
        &mut self,
        base_dir: &Path,
        entries: &[serde_json::Value],
        excludes: &[String],
    ) {
        for entry in entries {
            let Some(path_str) = entry.as_str() else {
                continue;
            };
            let path = base_dir.join(path_str);
            scan_classmap_path(&path, &mut self.fqn_to_path, excludes, &mut self.warnings);
        }
    }

    /// Read the `files` autoload entries and register files to always include.
    fn read_files(&mut self, base_dir: &Path, entries: &[serde_json::Value]) {
        for entry in entries {
            let Some(path_str) = entry.as_str() else {
                continue;
            };
            let path = base_dir.join(path_str);
            if path.is_file() {
                let canonical = path.canonicalize().unwrap_or(path);
                if !self.files_to_include.contains(&canonical) {
                    self.files_to_include.push(canonical);
                }
            }
        }
    }
}

/// Extract the path or paths from a JSON value (string or array of strings).
fn extract_paths(value: &serde_json::Value) -> Vec<&str> {
    match value {
        serde_json::Value::String(s) => vec![s.as_str()],
        serde_json::Value::Array(a) => a.iter().filter_map(|v| v.as_str()).collect(),
        _ => Vec::new(),
    }
}

// --- PSR-4 walker ---

/// Recursively walk a PSR-4 directory tree, mapping file paths to FQNs.
fn walk_psr4(dir: &Path, ns_prefix: &str, root: &Path, index: &mut HashMap<String, PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_psr4(&path, ns_prefix, root, index);
        } else if crate::source::is_discoverable_source_path(&path) {
            let Ok(rel) = path.strip_prefix(root) else {
                continue;
            };
            let mut parts: Vec<String> = rel
                .components()
                .filter_map(|c| match c {
                    std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
                    _ => None,
                })
                .collect();
            if parts.is_empty() {
                continue;
            }
            if let Some(last) = parts.last_mut() {
                *last = crate::source::discoverable_source_stem(last);
            }
            let suffix = parts.join("\\");
            let prefix = ns_prefix.trim_matches('\\');
            let fqn = if prefix.is_empty() {
                suffix
            } else {
                format!("{}\\{}", prefix, suffix)
            };
            let canonical = path.canonicalize().unwrap_or(path);
            index.entry(fqn).or_insert(canonical);
        }
    }
}

// --- PSR-0 walker ---

/// PSR-0 walker. Unlike PSR-4 (where the prefix is stripped before path
/// resolution), PSR-0 treats the directory as containing the full
/// namespace tree — class `Vendor\Pkg\Sub\Item` under prefix
/// `Vendor\Pkg\` mapping to `lib/` lives at `lib/Vendor/Pkg/Sub/Item.php`.
/// The walk derives the FQN directly from the path joined with `\`.
///
/// PSR-0 also supports underscore-style class names: when the prefix has
/// no `\` (e.g. `Twig_` → `lib/`), file `lib/Twig/Loader/Filesystem.php`
/// becomes class `Twig_Loader_Filesystem`. This is signalled by the
/// prefix not containing a backslash.
fn walk_psr0(dir: &Path, ns_prefix: &str, root: &Path, index: &mut HashMap<String, PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_psr0(&path, ns_prefix, root, index);
        } else if crate::source::is_discoverable_source_path(&path) {
            let Ok(rel) = path.strip_prefix(root) else {
                continue;
            };
            let mut parts: Vec<String> = rel
                .components()
                .filter_map(|c| match c {
                    std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
                    _ => None,
                })
                .collect();
            if parts.is_empty() {
                continue;
            }
            if let Some(last) = parts.last_mut() {
                *last = crate::source::discoverable_source_stem(last);
            }
            let prefix = ns_prefix.trim_matches('\\');
            let prefix_has_namespace = prefix.contains('\\');

            let fqn = if prefix_has_namespace {
                // Namespaced PSR-0: the directory tree mirrors the full
                // namespace path. Join components with `\`.
                parts.join("\\")
            } else {
                // Underscore-style PSR-0 (e.g. `Twig_` → `lib/`): every
                // path segment becomes part of the underscore-joined
                // class name.
                parts.join("_")
            };
            let canonical = path.canonicalize().unwrap_or(path);
            index.entry(fqn).or_insert(canonical);
        }
    }
}

// --- classmap scanner ---

/// Builds the warning reported for one classmap file the index could not read.
fn classmap_scan_warning(path: &Path, reason: &str) -> CompileWarning {
    // The diagnostic is about a file outside the entry program, so it has no position in it;
    // the message carries the path instead.
    CompileWarning::new(
        Span::dummy(),
        &format!(
            "autoload classmap: '{}' could not be indexed ({}); every class it declares stays \
             undefined and is never compiled",
            path.display(),
            reason
        ),
    )
}

/// Recursively scan a classmap path, descending into directories and
/// skipping excluded paths, then index all discovered PHP files.
fn scan_classmap_path(
    path: &Path,
    index: &mut HashMap<String, PathBuf>,
    excludes: &[String],
    warnings: &mut Vec<CompileWarning>,
) {
    if is_excluded(path, excludes) {
        return;
    }
    if path.is_file() {
        scan_classmap_file(path, index, warnings);
    } else if path.is_dir() {
        let Ok(entries) = std::fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            scan_classmap_path(&entry.path(), index, excludes, warnings);
        }
    }
}

/// Match `path` against the configured `exclude-from-classmap` glob
/// patterns. Returns true when any pattern matches.
fn is_excluded(path: &Path, excludes: &[String]) -> bool {
    if excludes.is_empty() {
        return false;
    }
    let canonical = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());
    let canonical_str = canonical.to_string_lossy();
    for pattern in excludes {
        if glob_match(pattern, &canonical_str) {
            return true;
        }
    }
    false
}

/// Resolve a user-provided pattern to an absolute glob string.
///
/// Relative patterns are joined with `base_dir` and canonicalised. A
/// trailing `/` is rewritten as `/**` so `"tests/"` matches everything
/// inside `tests/`, matching the manifest format's directory-shorthand semantic.
fn normalize_exclude_pattern(base_dir: &Path, raw: &str) -> String {
    let trimmed = raw.trim_start_matches("./");
    let with_dirstar = if trimmed.ends_with('/') {
        format!("{}**", trimmed)
    } else {
        trimmed.to_string()
    };
    if std::path::Path::new(&with_dirstar).is_absolute() {
        return with_dirstar;
    }
    let joined = base_dir.join(&with_dirstar);
    // Canonicalize the literal-prefix portion if possible, but keep glob
    // metacharacters intact afterwards. Splitting on the first wildcard
    // segment is the simplest way to do this without a full glob parser.
    let joined_str = joined.to_string_lossy().into_owned();
    if !joined_str.contains('*') && !joined_str.contains('?') {
        return joined
            .canonicalize()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or(joined_str);
    }
    let (literal_prefix, glob_tail) = split_at_first_wildcard(&joined_str);
    let canonical_prefix = std::path::Path::new(literal_prefix)
        .canonicalize()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| literal_prefix.to_string());
    if glob_tail.is_empty() {
        canonical_prefix
    } else {
        let sep = if canonical_prefix.ends_with('/') {
            ""
        } else {
            "/"
        };
        format!(
            "{}{}{}",
            canonical_prefix,
            sep,
            glob_tail.trim_start_matches('/')
        )
    }
}

/// Split a glob string at the first path segment that contains a
/// wildcard. Used to canonicalise the pure-literal prefix while leaving
/// the glob portion intact.
fn split_at_first_wildcard(input: &str) -> (&str, &str) {
    let mut last_slash = 0usize;
    for (idx, byte) in input.bytes().enumerate() {
        if byte == b'/' {
            last_slash = idx;
        } else if byte == b'*' || byte == b'?' {
            return input.split_at(last_slash);
        }
    }
    (input, "")
}

/// Glob match against a path. Supports:
///   `**` — match any sequence of characters, including `/`
///   `*`  — match any sequence of characters except `/`
///   `?`  — match a single character except `/`
///   any other character — literal
fn glob_match(pattern: &str, path: &str) -> bool {
    glob_match_bytes(pattern.as_bytes(), path.as_bytes())
}

/// Byte-level glob matcher called by `glob_match`. Handles `**`, `*`, `?`
/// meta-characters across path segments.
fn glob_match_bytes(p: &[u8], s: &[u8]) -> bool {
    let mut pi = 0;
    let mut si = 0;
    let mut backtrack: Option<(usize, usize)> = None;
    let mut star_double = false;
    while si < s.len() {
        if pi < p.len() && p[pi] == b'*' {
            // Detect `**` for cross-segment matching.
            star_double = pi + 1 < p.len() && p[pi + 1] == b'*';
            backtrack = Some((pi, si));
            if star_double {
                pi += 2;
                // Skip the optional trailing `/` that conventionally follows
                // `**` (e.g. `**/`).
                if pi < p.len() && p[pi] == b'/' {
                    pi += 1;
                }
            } else {
                pi += 1;
            }
            continue;
        }
        if pi < p.len() && (p[pi] == b'?' || p[pi] == s[si])
            && (p[pi] != b'?' || s[si] != b'/')
        {
            pi += 1;
            si += 1;
            continue;
        }
        if let Some((bp, bs)) = backtrack {
            // For `*`, expansion must not cross `/` characters.
            if !star_double && bs < s.len() && s[bs] == b'/' {
                return false;
            }
            backtrack = Some((bp, bs + 1));
            si = bs + 1;
            pi = if star_double { bp + 2 } else { bp + 1 };
            // Skip optional trailing `/` after `**` again.
            if star_double && pi < p.len() && p[pi] == b'/' {
                pi += 1;
            }
            continue;
        }
        return false;
    }
    while pi < p.len() && p[pi] == b'*' {
        pi += 1;
    }
    pi == p.len()
}

#[cfg(test)]
mod tests {
    use super::{glob_match, AutoloadIndex};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static MANIFEST_TEST_ID: AtomicUsize = AtomicUsize::new(0);

    /// Creates one isolated directory for structural manifest discovery tests.
    fn manifest_test_dir() -> std::path::PathBuf {
        let id = MANIFEST_TEST_ID.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "elephc_autoload_manifest_{}_{}",
            std::process::id(),
            id
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Verifies that manifest contents, rather than a tool-specific filename, select the adapter.
    #[test]
    fn structural_manifest_discovery_accepts_arbitrary_json_name() {
        let dir = manifest_test_dir();
        let entry_dir = dir.join("public");
        let source_dir = dir.join("src");
        std::fs::create_dir_all(&entry_dir).unwrap();
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(
            dir.join("module-layout.json"),
            r#"{"autoload":{"psr-4":{"Demo\\":"src/"}}}"#,
        )
        .unwrap();
        std::fs::write(
            source_dir.join("Thing.php"),
            "<?php namespace Demo; class Thing {}",
        )
        .unwrap();

        let index = AutoloadIndex::from_project_root(&entry_dir);
        assert_eq!(
            index.lookup("Demo\\Thing"),
            Some(source_dir.join("Thing.php").canonicalize().unwrap().as_path())
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Implements the `glob_literal` operation for this module.
    #[test]
    fn glob_literal() {
        assert!(glob_match("/a/b/c.php", "/a/b/c.php"));
        assert!(!glob_match("/a/b/c.php", "/a/b/d.php"));
    }

    /// Implements the `glob_star_within_segment` operation for this module.
    #[test]
    fn glob_star_within_segment() {
        assert!(glob_match("/a/*.php", "/a/foo.php"));
        assert!(!glob_match("/a/*.php", "/a/sub/foo.php"));
    }

    /// Implements the `glob_double_star_crosses_segments` operation for this module.
    #[test]
    fn glob_double_star_crosses_segments() {
        assert!(glob_match("/a/**/foo.php", "/a/foo.php"));
        assert!(glob_match("/a/**/foo.php", "/a/sub/foo.php"));
        assert!(glob_match("/a/**/foo.php", "/a/x/y/foo.php"));
    }

    /// Implements the `glob_directory_shorthand` operation for this module.
    #[test]
    fn glob_directory_shorthand() {
        assert!(glob_match("/a/tests/**", "/a/tests/foo.php"));
        assert!(glob_match("/a/tests/**", "/a/tests/sub/foo.php"));
        assert!(!glob_match("/a/tests/**", "/a/lib/foo.php"));
    }

    /// Implements the `glob_question_single_char` operation for this module.
    #[test]
    fn glob_question_single_char() {
        assert!(glob_match("/a/?.php", "/a/x.php"));
        assert!(!glob_match("/a/?.php", "/a/xx.php"));
        assert!(!glob_match("/a/?.php", "/a//.php"));
    }
}

/// Parses a PHP/LFC source file and indexes all class/interface/trait/enum declarations found.
///
/// A file this pass cannot read, lex, or parse contributes no classes, and every class it
/// declares then looks undefined to the rest of the compile — a demand for one of them resolves
/// to nothing and the class is simply never compiled. That is invisible at the use site, so each
/// failure is reported as a warning naming the file and the reason instead of being swallowed.
fn scan_classmap_file(
    path: &Path,
    index: &mut HashMap<String, PathBuf>,
    warnings: &mut Vec<CompileWarning>,
) {
    if !crate::source::is_discoverable_source_path(path) {
        return;
    }
    let content = match crate::source::read_physical_source(path) {
        Ok(content) => content,
        Err(error) => {
            warnings.push(classmap_scan_warning(path, &error.to_string()));
            return;
        }
    };
    let mode = crate::source::SourceMode::from_path(path);
    let tokens = match crate::lexer::tokenize_with_mode(&content, mode) {
        Ok(tokens) => tokens,
        Err(error) => {
            warnings.push(classmap_scan_warning(path, &error.message));
            return;
        }
    };
    let ast = match crate::parser::parse_with_mode(&tokens, mode) {
        Ok(ast) => ast,
        Err(error) => {
            warnings.push(classmap_scan_warning(path, &error.message));
            return;
        }
    };
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut current_namespace: Option<String> = None;
    for stmt in &ast {
        extract_classmap_decls(stmt, &mut current_namespace, &canonical, index);
    }
}

/// Recursively extract classmap declarations from a statement, tracking current namespace context.
fn extract_classmap_decls(
    stmt: &Stmt,
    current_namespace: &mut Option<String>,
    file_path: &Path,
    index: &mut HashMap<String, PathBuf>,
) {
    match &stmt.kind {
        StmtKind::NamespaceDecl { name } => {
            *current_namespace = name.as_ref().map(|n| n.as_canonical());
        }
        StmtKind::NamespaceBlock { name, body } => {
            let saved = current_namespace.clone();
            *current_namespace = name.as_ref().map(|n| n.as_canonical());
            for inner in body {
                extract_classmap_decls(inner, current_namespace, file_path, index);
            }
            *current_namespace = saved;
        }
        StmtKind::ClassDecl { name, .. }
        | StmtKind::InterfaceDecl { name, .. }
        | StmtKind::TraitDecl { name, .. }
        | StmtKind::EnumDecl { name, .. } => {
            let trimmed = name.trim_start_matches('\\');
            let fqn = match current_namespace.as_deref() {
                Some(ns) if !ns.is_empty() => {
                    format!("{}\\{}", ns.trim_start_matches('\\'), trimmed)
                }
                _ => trimmed.to_string(),
            };
            index.entry(fqn).or_insert_with(|| file_path.to_path_buf());
        }
        _ => {}
    }
}
