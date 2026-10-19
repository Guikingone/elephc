//! Purpose:
//! Names the installed Composer packages a test build must not autoload: `phpunit/phpunit`
//! and everything only it requires, read from `vendor/composer/installed.json`.
//!
//! Called from:
//! - `crate::pipeline::compile_inner` for an `elephc test` build, through
//!   `autoload::Registry::build_excluding`.
//!
//! Key details:
//! - The compiler provides `PHPUnit\Framework\TestCase` itself, so an installed PHPUnit is
//!   for php-src runs only. Left in the index, its `autoload.files` entry
//!   (`src/Framework/Assert/Functions.php`) is inlined into every build and drags the
//!   runner's own classes in through autoloading, where a runtime-dynamic `require` is a
//!   compile error. Excluding the package and its dependency closure keeps the suite's
//!   other dev dependencies (fakers, mocks) available.
//! - The closure walks the `require` maps recorded in `installed.json` so the list never
//!   has to name PHPUnit's dependencies by hand; without the file only `phpunit/phpunit`
//!   itself is excluded.

use std::collections::{HashSet, VecDeque};
use std::path::Path;

/// The package the compiler stands in for.
const PHPUNIT: &str = "phpunit/phpunit";

/// Returns `phpunit/phpunit` and every installed package reachable through its `require`
/// maps, as recorded in `<project_root>/vendor/composer/installed.json`.
pub fn phpunit_closure(project_root: &Path) -> HashSet<String> {
    let mut excluded = HashSet::from([PHPUNIT.to_string()]);
    let installed = project_root.join("vendor/composer/installed.json");
    let Ok(content) = std::fs::read_to_string(&installed) else {
        return excluded;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return excluded;
    };
    // Composer 2 writes `{"packages": [...]}`; Composer 1 wrote the bare array.
    let packages = json
        .get("packages")
        .and_then(|packages| packages.as_array())
        .or_else(|| json.as_array())
        .cloned()
        .unwrap_or_default();
    let mut queue = VecDeque::from([PHPUNIT.to_string()]);
    while let Some(name) = queue.pop_front() {
        let Some(package) = packages
            .iter()
            .find(|package| package.get("name").and_then(|n| n.as_str()) == Some(&name))
        else {
            continue;
        };
        let Some(requires) = package.get("require").and_then(|r| r.as_object()) else {
            continue;
        };
        for dependency in requires.keys() {
            // `php`, `ext-*` and `lib-*` are platform requirements, not packages.
            if !dependency.contains('/') {
                continue;
            }
            if excluded.insert(dependency.clone()) {
                queue.push_back(dependency.clone());
            }
        }
    }
    excluded
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Unit tests for the PHPUnit dependency closure read from `installed.json`.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Each test writes its own `vendor/composer/installed.json` under a scratch directory.

    use super::*;

    /// Writes `installed.json` with the given packages under a fresh scratch root.
    fn project_with(packages: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "elephc_dev_packages_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("vendor/composer")).unwrap();
        std::fs::write(root.join("vendor/composer/installed.json"), packages).unwrap();
        root
    }

    /// The closure follows `require` transitively, skips platform requirements, and leaves
    /// unrelated dev packages alone.
    #[test]
    fn walks_phpunit_requires_transitively() {
        let root = project_with(
            r#"{"packages": [
                {"name": "phpunit/phpunit", "require": {"php": ">=8.3", "ext-dom": "*", "sebastian/diff": "^6", "myclabs/deep-copy": "^1"}},
                {"name": "sebastian/diff", "require": {"sebastian/exporter": "^6"}},
                {"name": "sebastian/exporter", "require": {}},
                {"name": "myclabs/deep-copy", "require": {}},
                {"name": "fakerphp/faker", "require": {"php": ">=8.1"}}
            ], "dev-package-names": ["phpunit/phpunit", "sebastian/diff", "sebastian/exporter", "myclabs/deep-copy", "fakerphp/faker"]}"#,
        );
        let closure = phpunit_closure(&root);
        let mut sorted: Vec<&str> = closure.iter().map(String::as_str).collect();
        sorted.sort();
        assert_eq!(
            sorted,
            vec!["myclabs/deep-copy", "phpunit/phpunit", "sebastian/diff", "sebastian/exporter"]
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Without `installed.json` only PHPUnit itself is excluded.
    #[test]
    fn falls_back_to_phpunit_alone() {
        let root = std::env::temp_dir().join(format!("elephc_dev_packages_none_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        assert_eq!(phpunit_closure(&root), HashSet::from([PHPUNIT.to_string()]));
        let _ = std::fs::remove_dir_all(&root);
    }
}
