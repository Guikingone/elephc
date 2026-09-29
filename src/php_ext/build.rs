//! Purpose:
//! Runs an [`ExtensionBuildPlan`] against a real toolchain, producing the
//! extension's static archive and the Zend shim archive it resolves against.
//!
//! Called from:
//! - The hosted-extension install path, once the pinned PHP headers are staged.
//!
//! Key details:
//! - The shim is archived **separately**, once, not folded into each extension.
//!   Two hosted extensions would otherwise each carry a copy of every Zend
//!   symbol and collide at link time — the same reason `pcre2` ships
//!   `libelephc_pcre2_shim.a` beside `libpcre2-8.a`.
//! - The shim is C and is compiled with the C driver even when the extension is
//!   C++. Compiling it as C++ would mangle every Zend symbol, so nothing the
//!   extension calls would resolve.
//! - `NativeToolchain` selects only a C compiler, so the C++ driver is derived
//!   from it. An unrecognised compiler is an error rather than a guess: silently
//!   falling back to the C driver leaves the C++ runtime unresolved at link.
//! - Object names are index-prefixed. Extension tarballs put sources in
//!   subdirectories, so two `hash.c` in different directories share a stem, and
//!   naming objects by stem alone would let one overwrite the other — dropping a
//!   translation unit from the archive.

use std::fs;
use std::path::{Path, PathBuf};

use crate::native_deps::{run_checked, NativeError, NativeErrorKind, NativeToolchain};

use super::recipe::{compile_args, ExtensionBuildPlan, Language, TranslationUnit};
use super::shim;

/// Fixed name of the shared Zend shim archive. One per program, not per extension.
pub const SHIM_ARCHIVE: &str = "libelephc_zend_shim.a";

/// Everything one hosted-extension build needs beyond its plan.
pub struct BuildRequest<'a> {
    /// Extension name, used in diagnostics.
    pub extension: &'a str,
    pub plan: &'a ExtensionBuildPlan,
    pub toolchain: &'a NativeToolchain,
    /// Scratch directory for object files and generated shim sources.
    pub work_dir: &'a Path,
}

/// C driver names and the C++ driver that belongs with each.
const CXX_FOR: &[(&str, &str)] = &[("clang", "clang++"), ("gcc", "g++"), ("cc", "c++")];

/// Derives the C++ driver that pairs with a selected C compiler.
///
/// Handles cross tuples (`aarch64-linux-gnu-gcc`) and version suffixes
/// (`clang-18`) by substituting the driver component in place, leaving the
/// directory and every other component untouched.
pub fn cxx_driver(cc: &Path) -> Result<PathBuf, NativeError> {
    let name = cc
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| unpaired_driver(&cc.display().to_string()))?;
    let mut parts: Vec<&str> = name.split('-').collect();
    let position = parts
        .iter()
        .rposition(|part| CXX_FOR.iter().any(|(c, _)| c == part))
        .ok_or_else(|| unpaired_driver(name))?;
    let (_, cxx) = CXX_FOR
        .iter()
        .find(|(c, _)| *c == parts[position])
        .expect("rposition matched a known driver");
    parts[position] = cxx;
    Ok(cc.with_file_name(parts.join("-")))
}

/// Reports a compiler no C++ driver can be derived from.
fn unpaired_driver(name: &str) -> NativeError {
    NativeError::new(
        NativeErrorKind::Toolchain,
        format!(
            "cannot derive a C++ driver from compiler '{name}'; a hosted extension with C++ \
             sources needs one, and linking it with the C driver would leave the C++ runtime \
             unresolved. Set CXX-capable tools or host a C-only extension"
        ),
    )
}

/// Picks the driver one translation unit must be compiled with.
fn driver_for(unit: &TranslationUnit, toolchain: &NativeToolchain) -> Result<PathBuf, NativeError> {
    match unit.language {
        Language::C => Ok(toolchain.cc.clone()),
        Language::Cxx => cxx_driver(&toolchain.cc),
    }
}

/// Names one object uniquely within a build, since source stems repeat across
/// an extension's subdirectories.
fn object_name(index: usize, source: &Path) -> String {
    let stem = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("unit");
    format!("{index:04}-{stem}.o")
}

/// Compile arguments for a shim unit.
///
/// Deliberately not [`compile_args`]: the plan's defines include
/// `HAVE_CONFIG_H`, which makes a translation unit pull in the *extension's*
/// generated `config.h`. The shim has no business including that.
fn shim_compile_args(include_dirs: &[PathBuf], source: &Path, object: &Path) -> Vec<String> {
    let mut args = vec!["-fPIC".to_string()];
    for dir in include_dirs {
        args.push("-I".to_string());
        args.push(dir.display().to_string());
    }
    args.push("-c".to_string());
    args.push(source.display().to_string());
    args.push("-o".to_string());
    args.push(object.display().to_string());
    args
}

/// Archives that must be handed to the linker, in an order where every
/// reference precedes its definition: extensions call into the shim, so the
/// shim is last.
pub fn link_order(extension_archives: &[PathBuf], shim_archive: &Path) -> Vec<PathBuf> {
    let mut ordered = extension_archives.to_vec();
    ordered.push(shim_archive.to_path_buf());
    ordered
}

/// Writes the embedded shim sources into a directory and returns their paths.
fn stage_shim(work_dir: &Path) -> Result<Vec<PathBuf>, NativeError> {
    let mut written = Vec::with_capacity(shim::ALL.len());
    for (name, source) in shim::ALL {
        let path = work_dir.join(name);
        fs::write(&path, source)
            .map_err(|error| NativeError::io("write Zend shim source", &path, error))?;
        written.push(path);
    }
    Ok(written)
}

/// Compiles the Zend shim once into its own archive.
///
/// Callers build this a single time per program and share it across every
/// hosted extension.
pub fn build_shim(
    toolchain: &NativeToolchain,
    include_dirs: &[PathBuf],
    work_dir: &Path,
    archive: &Path,
) -> Result<PathBuf, NativeError> {
    create_dir(work_dir, "create Zend shim build directory")?;
    let sources = stage_shim(work_dir)?;
    let mut objects = Vec::with_capacity(sources.len());
    for (index, source) in sources.iter().enumerate() {
        let object = work_dir.join(object_name(index, source));
        // Always the C driver: the shim defines the symbols the extension
        // resolves against, and C++ would mangle every one of them.
        let mut compile = toolchain.command(&toolchain.cc);
        compile.args(shim_compile_args(include_dirs, source, &object));
        run_checked(&mut compile, "compile Zend shim")?;
        objects.push(object);
    }
    archive_objects(toolchain, archive, &objects, "Zend shim")?;
    remove_all(sources.iter().chain(objects.iter()))?;
    Ok(archive.to_path_buf())
}

/// Compiles one hosted extension into its own archive, without the shim.
pub fn build_extension(request: &BuildRequest<'_>) -> Result<PathBuf, NativeError> {
    // Checked before anything is created: a plan whose sources were all filtered
    // out would otherwise archive nothing, link cleanly, and provide no functions.
    if request.plan.units.is_empty() {
        return Err(NativeError::new(
            NativeErrorKind::Build,
            format!(
                "hosted extension '{}' has no C or C++ sources to compile",
                request.extension
            ),
        ));
    }
    create_dir(request.work_dir, "create hosted extension build directory")?;
    let mut objects = Vec::with_capacity(request.plan.units.len());
    for (index, unit) in request.plan.units.iter().enumerate() {
        let object = request.work_dir.join(object_name(index, &unit.source));
        let driver = driver_for(unit, request.toolchain)?;
        let mut compile = request.toolchain.command(&driver);
        compile.args(compile_args(request.plan, unit, &object));
        run_checked(
            &mut compile,
            &format!("compile hosted extension '{}'", request.extension),
        )?;
        objects.push(object);
    }
    archive_objects(
        request.toolchain,
        &request.plan.archive,
        &objects,
        request.extension,
    )?;
    remove_all(objects.iter())?;
    Ok(request.plan.archive.clone())
}

/// Archives objects, indexes the result, and proves the archiver can read it back.
fn archive_objects(
    toolchain: &NativeToolchain,
    archive: &Path,
    objects: &[PathBuf],
    what: &str,
) -> Result<(), NativeError> {
    if let Some(parent) = archive.parent() {
        create_dir(parent, "create hosted extension library directory")?;
    }
    // `crs` replaces rather than appends, so a rebuild cannot leave a stale
    // object from a source that has since been removed.
    let mut create = toolchain.command(&toolchain.ar);
    create.arg("crs").arg(archive).args(objects);
    run_checked(&mut create, &format!("archive {what}"))?;
    let mut index = toolchain.command(&toolchain.ranlib);
    index.arg(archive);
    run_checked(&mut index, &format!("index {what} archive"))?;
    let mut inspect = toolchain.command(&toolchain.ar);
    inspect.arg("t").arg(archive);
    run_checked(&mut inspect, &format!("validate {what} archive"))?;
    Ok(())
}

/// Creates a directory, reporting the path that failed.
fn create_dir(path: &Path, action: &str) -> Result<(), NativeError> {
    fs::create_dir_all(path).map_err(|error| NativeError::io(action, path, error))
}

/// Removes build intermediates, failing loudly rather than leaving them behind.
fn remove_all<'a>(paths: impl Iterator<Item = &'a PathBuf>) -> Result<(), NativeError> {
    for path in paths {
        fs::remove_file(path)
            .map_err(|error| NativeError::io("remove build intermediate", path, error))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::php_ext::recipe::plan_build;

    fn paths(items: &[&str]) -> Vec<PathBuf> {
        items.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn derives_the_cxx_driver_for_each_known_compiler() {
        for (c, cxx) in [("clang", "clang++"), ("gcc", "g++"), ("cc", "c++")] {
            assert_eq!(
                cxx_driver(Path::new(c)).expect("known compiler"),
                PathBuf::from(cxx)
            );
        }
    }

    /// Cross builds select a tuple-prefixed compiler; only the driver component
    /// may change, and the directory must survive.
    #[test]
    fn cxx_driver_preserves_the_tuple_and_directory() {
        assert_eq!(
            cxx_driver(Path::new("/opt/cross/bin/aarch64-unknown-linux-gnu-gcc")).expect("tuple"),
            PathBuf::from("/opt/cross/bin/aarch64-unknown-linux-gnu-g++")
        );
    }

    /// Debian-style versioned names put the version after the driver.
    #[test]
    fn cxx_driver_handles_a_version_suffix() {
        assert_eq!(
            cxx_driver(Path::new("/usr/bin/clang-18")).expect("versioned"),
            PathBuf::from("/usr/bin/clang++-18")
        );
    }

    /// Guessing here would surface as a wall of unresolved C++ runtime symbols
    /// at link time, far from the cause.
    #[test]
    fn an_unpaired_compiler_is_refused_rather_than_guessed() {
        let error = cxx_driver(Path::new("/usr/bin/tcc")).expect_err("no C++ driver");
        assert!(
            error.to_string().contains("cannot derive a C++ driver"),
            "unexpected message: {error}"
        );
    }

    /// Same stem, different directories — real extension layouts do this, and a
    /// stem-only object name would silently drop one translation unit.
    #[test]
    fn objects_from_same_named_sources_do_not_collide() {
        let first = object_name(0, Path::new("src/php/hash.c"));
        let second = object_name(1, Path::new("src/ds/hash.c"));
        assert_ne!(first, second);
    }

    /// The shim defines what the extension references, so it must be linked last.
    #[test]
    fn the_shim_archive_is_linked_after_every_extension() {
        let ordered = link_order(
            &paths(&["libapcu.a", "libds.a"]),
            Path::new("libelephc_zend_shim.a"),
        );
        assert_eq!(ordered.last(), Some(&PathBuf::from("libelephc_zend_shim.a")));
        assert_eq!(ordered.len(), 3);
    }

    /// Folding the shim into each extension would give two hosted extensions two
    /// copies of every Zend symbol.
    #[test]
    fn the_shim_is_archived_once_not_per_extension() {
        let ordered = link_order(&paths(&["libapcu.a", "libds.a"]), Path::new(SHIM_ARCHIVE));
        let shim_copies = ordered
            .iter()
            .filter(|archive| archive.ends_with(SHIM_ARCHIVE))
            .count();
        assert_eq!(shim_copies, 1);
    }

    /// HAVE_CONFIG_H would make the shim include the extension's generated
    /// config.h, which describes the extension and not the engine.
    #[test]
    fn shim_units_do_not_inherit_the_extension_defines() {
        let plan = plan_build(
            &paths(&["php_apc.c"]),
            &paths(&["/php/include"]),
            PathBuf::from("libapcu.a"),
        );
        let extension_args = compile_args(&plan, &plan.units[0], Path::new("a.o"));
        assert!(extension_args.contains(&"-DHAVE_CONFIG_H".to_string()));

        let shim_args = shim_compile_args(
            &paths(&["/php/include"]),
            Path::new("elephc_zend_core.c"),
            Path::new("core.o"),
        );
        assert!(!shim_args.iter().any(|arg| arg.starts_with("-DHAVE_CONFIG_H")));
        assert!(
            shim_args.contains(&"/php/include".to_string()),
            "the shim still needs the engine headers it implements against"
        );
    }

    /// Compiling the shim as C++ would mangle every symbol the extension calls.
    #[test]
    fn shim_units_are_c_even_beside_a_cxx_extension() {
        let unit = TranslationUnit {
            source: PathBuf::from("elephc_zend_core.c"),
            language: Language::C,
        };
        let cxx_unit = TranslationUnit {
            source: PathBuf::from("php_simdjson.cpp"),
            language: Language::Cxx,
        };
        let toolchain = probe_toolchain("/usr/bin/clang");
        assert_eq!(
            driver_for(&unit, &toolchain).expect("C driver"),
            PathBuf::from("/usr/bin/clang")
        );
        assert_eq!(
            driver_for(&cxx_unit, &toolchain).expect("C++ driver"),
            PathBuf::from("/usr/bin/clang++")
        );
    }

    /// A plan whose sources were all filtered out must not produce an empty
    /// archive that links cleanly and provides nothing.
    #[test]
    fn an_extension_with_no_sources_is_a_build_error() {
        let plan = plan_build(&paths(&["README.md"]), &[], PathBuf::from("libx.a"));
        assert!(plan.units.is_empty());
        let toolchain = probe_toolchain("/usr/bin/clang");
        let request = BuildRequest {
            extension: "x",
            plan: &plan,
            toolchain: &toolchain,
            work_dir: Path::new("/nonexistent-elephc-php-ext-probe"),
        };
        let error = build_extension(&request).expect_err("nothing to compile");
        assert!(
            error.to_string().contains("no C or C++ sources"),
            "unexpected message: {error}"
        );
    }

    /// Builds a toolchain value for driver-selection tests. Nothing here runs a
    /// compiler; only `cc` is consulted.
    fn probe_toolchain(cc: &str) -> NativeToolchain {
        use crate::native_deps::ToolIdentity;
        let identity = ToolIdentity {
            command: cc.to_string(),
            version: "probe".to_string(),
        };
        NativeToolchain {
            cc: PathBuf::from(cc),
            ar: PathBuf::from("/usr/bin/ar"),
            ranlib: PathBuf::from("/usr/bin/ranlib"),
            target_tuple: "aarch64-apple-darwin".to_string(),
            abi: "darwin".to_string(),
            fingerprint: "probe".to_string(),
            compiler: identity.clone(),
            archiver: identity.clone(),
            ranlib_identity: identity,
        }
    }
}
