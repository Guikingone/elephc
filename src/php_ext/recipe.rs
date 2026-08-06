//! Purpose:
//! Works out how a hosted PHP extension must be compiled — which sources, which
//! compiler, which flags — as a value that can be inspected and tested, separate
//! from the act of running the commands.
//!
//! Called from:
//! - The curated recipe that builds a declared extension into a staticlib.
//!
//! Key details:
//! - Planning is pure. `native_deps::recipes::pcre2` interleaves deciding and
//!   executing, so its decisions cannot be asserted without invoking a compiler;
//!   here the plan is a value and only [`ExtensionBuildPlan::commands`] touches
//!   the toolchain.
//! - An extension with any C++ translation unit must be linked by the C++ driver,
//!   otherwise its runtime support is missing at link time.
//! - The include path is supplied by the caller. Extensions are compiled against
//!   headers Elephc controls rather than a local PHP installation — that is what
//!   removes per-version binary-layout skew — but *which* headers those are is a
//!   separate decision, so this module does not assume a source for them.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Which compiler driver a translation unit needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Language {
    C,
    Cxx,
}

/// One translation unit and the driver it requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationUnit {
    pub source: PathBuf,
    pub language: Language,
}

/// A complete, inspectable description of how to build one extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionBuildPlan {
    pub units: Vec<TranslationUnit>,
    pub include_dirs: Vec<PathBuf>,
    pub defines: Vec<String>,
    pub archive: PathBuf,
}

impl ExtensionBuildPlan {
    /// True when any unit is C++, which decides the link driver.
    pub fn needs_cxx_driver(&self) -> bool {
        self.units.iter().any(|u| u.language == Language::Cxx)
    }
}

/// Classifies a source file, ignoring anything that is not a translation unit.
fn classify(path: &Path) -> Option<Language> {
    match path.extension()?.to_str()? {
        "c" => Some(Language::C),
        "cpp" | "cc" | "cxx" | "c++" => Some(Language::Cxx),
        _ => None,
    }
}

/// Builds a plan from an extension's source files.
///
/// `sources` is every candidate path; non-source files are ignored rather than
/// rejected, since extension tarballs carry headers, tests and build scripts.
/// Ordering is deterministic so a rebuild produces an identical command list.
pub fn plan_build(
    sources: &[PathBuf],
    include_dirs: &[PathBuf],
    archive: PathBuf,
) -> ExtensionBuildPlan {
    let mut seen = BTreeSet::new();
    let mut units: Vec<TranslationUnit> = sources
        .iter()
        .filter_map(|path| {
            let language = classify(path)?;
            if !seen.insert(path.clone()) {
                return None;
            }
            Some(TranslationUnit {
                source: path.clone(),
                language,
            })
        })
        .collect();
    units.sort_by(|a, b| a.source.cmp(&b.source));

    ExtensionBuildPlan {
        units,
        include_dirs: include_dirs.to_vec(),
        defines: vec![
            // Extensions test this to know they are a loadable module rather
            // than part of a statically-built php binary.
            "HAVE_CONFIG_H".to_string(),
            "ZEND_COMPILE_DL_EXT=1".to_string(),
        ],
        archive,
    }
}

/// Renders the compile arguments for one unit, in a fixed order so that two
/// plans compare equal iff they would run the same command.
pub fn compile_args(plan: &ExtensionBuildPlan, unit: &TranslationUnit, object: &Path) -> Vec<String> {
    let mut args = Vec::new();
    args.push("-fPIC".to_string());
    for define in &plan.defines {
        args.push(format!("-D{define}"));
    }
    for dir in &plan.include_dirs {
        args.push("-I".to_string());
        args.push(dir.display().to_string());
    }
    args.push("-c".to_string());
    args.push(unit.source.display().to_string());
    args.push("-o".to_string());
    args.push(object.display().to_string());
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(items: &[&str]) -> Vec<PathBuf> {
        items.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn selects_c_and_cxx_sources_and_ignores_the_rest() {
        // A real extension tarball: sources next to headers, stubs and scripts.
        let sources = paths(&[
            "php_apc.c",
            "apc_cache.c",
            "php_apc.h",
            "php_apc.stub.php",
            "config.m4",
            "README.md",
        ]);
        let plan = plan_build(&sources, &paths(&["/inc"]), PathBuf::from("libapcu.a"));
        assert_eq!(plan.units.len(), 2);
        assert!(plan.units.iter().all(|u| u.language == Language::C));
    }

    #[test]
    fn recognises_every_cxx_extension_spelling() {
        for name in ["a.cpp", "b.cc", "c.cxx"] {
            let plan = plan_build(&paths(&[name]), &[], PathBuf::from("x.a"));
            assert_eq!(
                plan.units.first().map(|u| u.language),
                Some(Language::Cxx),
                "{name} must be treated as C++"
            );
        }
    }

    /// simdjson is C++ with C translation units alongside; linking it with the C
    /// driver would leave the C++ runtime unresolved.
    #[test]
    fn a_single_cxx_unit_forces_the_cxx_driver() {
        let mixed = plan_build(
            &paths(&["php_simdjson.cpp", "helper.c"]),
            &[],
            PathBuf::from("libsimdjson.a"),
        );
        assert!(mixed.needs_cxx_driver());

        let pure_c = plan_build(&paths(&["php_apc.c"]), &[], PathBuf::from("libapcu.a"));
        assert!(!pure_c.needs_cxx_driver());
    }

    #[test]
    fn ordering_is_deterministic_regardless_of_input_order() {
        let one = plan_build(&paths(&["b.c", "a.c", "c.c"]), &[], PathBuf::from("x.a"));
        let two = plan_build(&paths(&["c.c", "b.c", "a.c"]), &[], PathBuf::from("x.a"));
        assert_eq!(one, two, "a rebuild must produce an identical command list");
    }

    #[test]
    fn duplicate_sources_are_compiled_once() {
        let plan = plan_build(&paths(&["a.c", "a.c"]), &[], PathBuf::from("x.a"));
        assert_eq!(plan.units.len(), 1);
    }

    #[test]
    fn compile_args_carry_includes_and_defines() {
        let plan = plan_build(
            &paths(&["php_apc.c"]),
            &paths(&["/zend/include", "/ext/include"]),
            PathBuf::from("libapcu.a"),
        );
        let args = compile_args(&plan, &plan.units[0], Path::new("php_apc.o"));
        assert!(args.contains(&"-fPIC".to_string()), "objects go into an archive");
        assert!(args.contains(&"-DZEND_COMPILE_DL_EXT=1".to_string()));
        assert!(args.contains(&"/zend/include".to_string()));
        assert!(args.contains(&"/ext/include".to_string()));
        let include_flags = args.iter().filter(|a| *a == "-I").count();
        assert_eq!(include_flags, 2, "one -I per include directory");
    }

    /// The include path is what makes hosting version-proof, so an empty one is
    /// a caller error worth seeing in a test rather than a silent miscompile.
    #[test]
    fn a_plan_without_includes_produces_no_include_flags() {
        let plan = plan_build(&paths(&["a.c"]), &[], PathBuf::from("x.a"));
        let args = compile_args(&plan, &plan.units[0], Path::new("a.o"));
        assert!(!args.iter().any(|a| a == "-I"));
    }
}
