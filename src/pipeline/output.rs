//! Purpose:
//! Computes compilation artifact paths and post-link capability warnings.
//!
//! Called from:
//! - `crate::pipeline::compile()` and its backend stage.
//!
//! Key details:
//! - Shared-library names follow each target platform's conventional suffix.

use std::path::{Path, PathBuf};

use crate::codegen::platform::{Platform, Target};
use crate::codegen::{Emit, RuntimeFeatures};

/// Holds the paths for all compilation output files (assembly, object, binary, source map).
pub(super) struct OutputPaths {
    pub(super) asm: PathBuf,
    pub(super) obj: PathBuf,
    pub(super) bin: PathBuf,
    pub(super) source_map: PathBuf,
    /// NPM packaging (wasm-only) writes into `<stem>-npm/`; `None` everywhere else.
    pub(super) package_dir: Option<PathBuf>,
}

/// Returns the post-link reminder for dynamic eval without optional regex support.
pub(super) fn dynamic_eval_capability_warning(
    runtime_features: RuntimeFeatures,
) -> Option<&'static str> {
    (runtime_features.eval_bridge && !runtime_features.regex).then_some(concat!(
        "warning: dynamic eval was compiled without optional regex support\n",
        "evaluated code that uses preg_* or mb_ereg_match() will fail at runtime; enable it with:\n",
        "  elephc native add pcre2\n",
        "  elephc --with-regex <source-file>",
    ))
}

/// Computes output paths for .s (assembly), .o (object), binary, and .map (source map) files
/// derived from the input filename.
///
/// Executable mode produces `<stem>` (no extension). Cdylib mode produces
/// `lib<stem>.so` (Linux) or `lib<stem>.dylib` (macOS), matching the conventional
/// shared-library naming that `dlopen(3)` and linker `-l` flags expect.
pub(super) fn output_paths(filename: &str, target: Target, emit: Emit) -> OutputPaths {
    let path = Path::new(filename);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let parent = path.parent().unwrap_or(Path::new("."));

    // The WebAssembly target emits a `.wat` text module (the readable form, the
    // analogue of `.s`) and a `.wasm` binary (the artifact). It never produces a
    // `.o` object or runs the native linker. NPM packaging writes into a
    // `<stem>-npm/` directory to avoid colliding with the ordinary binary path.
    if target.is_wasm() {
        let package_dir =
            matches!(emit, Emit::NpmPackage).then(|| parent.join(format!("{}-npm", stem)));
        let bin = package_dir
            .as_ref()
            .map(|dir| dir.join("module.wasm"))
            .unwrap_or_else(|| parent.join(format!("{}.wasm", stem)));
        return OutputPaths {
            asm: parent.join(format!("{}.wat", stem)),
            obj: parent.join(format!("{}.o", stem)),
            bin,
            source_map: parent.join(format!("{}.map", stem)),
            package_dir,
        };
    }

    let bin_name = match emit {
        // NpmPackage is wasm-only; for any native build it never occurs, but keep
        // the match exhaustive by treating it like an executable.
        Emit::Executable | Emit::NpmPackage => stem.to_string(),
        Emit::Cdylib => match target.platform {
            Platform::MacOS => format!("lib{}.dylib", stem),
            Platform::Linux => format!("lib{}.so", stem),
            Platform::Windows => panic!("Windows target is not yet supported (see issue #379)"),
        },
    };
    OutputPaths {
        asm: parent.join(format!("{}.s", stem)),
        obj: parent.join(format!("{}.o", stem)),
        bin: parent.join(bin_name),
        source_map: parent.join(format!("{}.map", stem)),
        package_dir: None,
    }
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Unit tests for target-specific compilation output path selection.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - These tests only calculate paths and never write compilation artifacts.

    use super::output_paths;
    use crate::codegen::platform::Target;
    use crate::codegen::Emit;
    use std::path::Path;

    /// Verifies ordinary WASM output remains adjacent to the PHP source.
    #[test]
    fn wasm_executable_uses_wat_and_wasm_paths() {
        let paths = output_paths("examples/hello.php", Target::wasm(), Emit::Executable);
        assert_eq!(paths.asm, Path::new("examples/hello.wat"));
        assert_eq!(paths.bin, Path::new("examples/hello.wasm"));
        assert_eq!(paths.package_dir, None);
    }

    /// Verifies NPM output uses an isolated directory containing `module.wasm`.
    #[test]
    fn wasm_npm_uses_isolated_package_directory() {
        let paths = output_paths("examples/hello.php", Target::wasm(), Emit::NpmPackage);
        assert_eq!(
            paths.package_dir.as_deref(),
            Some(Path::new("examples/hello-npm"))
        );
        assert_eq!(paths.bin, Path::new("examples/hello-npm/module.wasm"));
    }
}
