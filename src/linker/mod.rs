//! Purpose:
//! Provides the assembler/linker facade used by the compiler pipeline.
//! Adapts legacy string options into a typed plan and orchestrates platform helpers.
//!
//! Called from:
//! - `crate::pipeline::compile()` after user and runtime object generation.
//! - `crate::cli` for table-driven bridge flag validation.
//!
//! Key details:
//! - The legacy `link()` API remains behavior-compatible while callers migrate to `LinkPlan`.
//! - Dependency resolution stays outside this module; the linker consumes typed paths only.

mod archive_dedup;
mod aarch64_encode;
mod asm_cache;
mod asm_split;

mod bridges;
mod command;
mod pdo;
mod sdk;

use std::path::{Path, PathBuf};
use std::process::{self, Command};

use crate::codegen::platform::{AppleVariant, Platform, Target};
use crate::codegen::Emit;
use crate::link_plan::{LinkItem, LinkPlan};

use self::command::{LinkPaths, MacSdk};

/// Structured failure produced while preparing typed linker inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkError {
    /// A requested Elephc bridge archive could not be resolved to a regular file.
    MissingBridge {
        /// Authoritative bridge linker name that could not be materialized.
        name: String,
    },
}

impl std::fmt::Display for LinkError {
    /// Formats an actionable linker-preparation diagnostic.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingBridge { name } => {
                write!(formatter, "required Elephc bridge `{name}` could not be found")
            }
        }
    }
}

impl std::error::Error for LinkError {}

/// Resolves a `--with-<flag>` suffix to its bridge linker library name.
pub(crate) fn bridge_lib_for_flag(flag: &str) -> Option<&'static str> {
    bridges::bridge_lib_for_flag(flag)
}

/// Returns every accepted `--with-<flag>` suffix in bridge table order.
pub(crate) fn crate_flag_names() -> Vec<&'static str> {
    bridges::crate_flag_names()
}

/// Maps a `--with-<flag>` suffix to the archive filename it resolves to.
pub(crate) fn archive_filename_for_flag(flag: &str) -> Option<String> {
    bridges::archive_filename_for_flag(flag)
}

/// Returns bridge library/flag pairs present in one planned named-library set.
pub(crate) fn bridges_in(
    link_libraries: &[String],
) -> Vec<(&'static str, &'static str)> {
    bridges::bridges_in(link_libraries)
}

/// Maps one bridge library name to the PHP extensions identified by linking that bridge.
pub(crate) fn php_extensions_for_lib(lib_name: &str) -> &'static [&'static str] {
    bridges::php_extensions_for_lib(lib_name)
}

/// Returns native libraries required by the selected optional PDO bridge profile.
pub(crate) fn pdo_system_libraries() -> Vec<&'static str> {
    pdo::system_libraries()
}

/// Builds the assembler invocation for a target, minus the input and output.
///
/// A Mach-O object records the platform it was built for, and `ld` refuses to
/// link a macOS-tagged object into an iOS image — the failure reads
/// *"building for 'iOS-simulator', but linking in object file built for
/// 'macOS'"*. Plain `as` has no way to say "iOS", so non-macOS Apple targets
/// assemble through `clang`, which stamps the platform from `-target` and needs
/// the matching SDK to resolve it.
///
/// Shared by the user object and the cached runtime object. Both must carry the
/// same platform or the link fails on whichever one disagrees, and they used to
/// build this command separately.
pub(crate) fn assembler_command(target: Target) -> Command {
    if target.platform == Platform::MacOS && target.apple_variant != AppleVariant::MacOS {
        let sdk_path = sdk::macos_sdk_path(target.apple_sdk_name());
        let mut assembler = Command::new("clang");
        assembler
            .arg("-c")
            .args(["-target", &target.apple_clang_triple()])
            .args(["-isysroot", &sdk_path]);
        return assembler;
    }
    let mut assembler = Command::new(target.assembler_cmd());
    if target.platform == Platform::MacOS {
        assembler.args(["-arch", target.darwin_arch_name()]);
    }
    assembler
}

/// Invokes the target assembler for one generated assembly source file.
pub(crate) fn assemble(target: Target, asm_path: &Path, obj_path: &Path) {
    let mut assembler = assembler_command(target);
    assembler.arg("-o").arg(obj_path).arg(asm_path);
    command::run_tool("Assembler", &mut assembler);
}

/// Ceiling on concurrent `as` processes for one user object.
///
/// The default is one per core. It used to stop at eight, where the curve had flattened while
/// the slices were rendered one after another before any assembler started; with every slice
/// rendered and assembled by its own worker, the Symfony `--web` assembly phase measured 11.7 s
/// at 12 jobs, 8.5 s at 18 (this machine's core count) and 8.4 s at 24 and 36. Assembler memory
/// is proportional to the slice, so more, smaller slices do not add to it.
const MAX_ASSEMBLER_JOBS: usize = 64;

/// Objects produced from one user assembly file, in slice order, plus a note for `--timings`.
pub(crate) struct AssembledObjects {
    /// Object paths in the order the slices must reach the link line. The first
    /// entry is always the caller's `obj_path`, so every existing cleanup and
    /// debug-map path keeps working unchanged.
    pub(crate) objects: Vec<PathBuf>,
    /// What the split had to publish, when it split at all.
    pub(crate) note: Option<String>,
}

/// Returns how many `as` processes may assemble one user object.
///
/// `ELEPHC_ASM_JOBS` overrides the host-derived default, and `ELEPHC_ASM_JOBS=1`
/// is the identity setting: it takes the single-`as` path below, so the `.s` and
/// the object are byte-for-byte what they are today.
fn assembler_jobs() -> usize {
    if let Ok(value) = std::env::var("ELEPHC_ASM_JOBS") {
        return value.trim().parse::<usize>().unwrap_or(1).clamp(1, 64);
    }
    std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
        .clamp(1, MAX_ASSEMBLER_JOBS)
}

/// Assembles one user assembly file, splitting it across parallel `as` runs when
/// the target supports it, and returns the objects in slice order.
///
/// Every fallback path here produces exactly today's single object: an
/// unsupported target or artifact, `ELEPHC_ASM_JOBS=1`, `--debug-info` (whose
/// dSYM debug map has to name one object per compilation unit and is not worth
/// re-deriving for a build the developer is about to step through), an
/// unreadable `.s`, an assembly with no legal cut, or a failed slice write.
///
/// The source text is freed as soon as the split is done, and each slice once it is written, so
/// the assemblers run beside the compiler's residual heap instead of the text plus every slice.
pub(crate) fn assemble_parallel(
    target: Target,
    emit: Emit,
    debug_info: bool,
    asm_path: &Path,
    obj_path: &Path,
) -> AssembledObjects {
    let single = || AssembledObjects {
        objects: vec![obj_path.to_path_buf()],
        note: None,
    };
    let jobs = assembler_jobs();
    if jobs <= 1 || debug_info || !asm_split::supports_split(target, emit) {
        assemble(target, asm_path, obj_path);
        return single();
    }
    // Map the `.s` instead of reading it: the pages are clean and file-backed, so the kernel can
    // drop them under pressure instead of the build holding a 1.5 GB private copy of the text.
    let Some(mapped) = crate::mapped_file::MappedFile::open(asm_path) else {
        assemble(target, asm_path, obj_path);
        return single();
    };
    let Ok(source) = std::str::from_utf8(mapped.bytes()) else {
        drop(mapped);
        assemble(target, asm_path, obj_path);
        return single();
    };

    let slice_asm = |index: usize| obj_path.with_extension(format!("slice{index}.s"));
    let slice_obj = |index: usize| match index {
        0 => obj_path.to_path_buf(),
        _ => obj_path.with_extension(format!("slice{index}.o")),
    };

    let Some(plan) = asm_split::plan_split(
        source,
        jobs,
        target.platform.local_label_prefix(),
        target.arch == crate::codegen::platform::Arch::AArch64 && encode_enabled(),
    ) else {
        drop(mapped);
        assemble(target, asm_path, obj_path);
        return single();
    };
    let slice_count = plan.slices();

    // Every slice is rendered, keyed, written and -- unless the cache already holds its object --
    // assembled by a worker of its own. Rendering every slice before starting any assembler, as
    // this did, left the assemblers idle for the whole serial render; now the first `as` starts
    // as soon as its own slice is on disk, and no slice ever exists in memory whole: its cache
    // key is folded from the same chunks that are written.
    let caching = asm_cache::is_enabled();
    let identity = caching.then(|| asm_cache::assembler_identity(&assembler_command(target)));
    let outcomes: Vec<std::thread::Result<std::io::Result<SliceOutcome>>> =
        std::thread::scope(|scope| {
            let workers: Vec<_> = (0..slice_count)
                .map(|index| {
                    let (plan, identity) = (&plan, identity.as_deref());
                    scope.spawn(move || -> std::io::Result<SliceOutcome> {
                        let asm = slice_asm(index);
                        let mut key = identity.map(asm_cache::slice_key_seed);
                        let file = std::fs::File::create(&asm)?;
                        let mut writer = std::io::BufWriter::with_capacity(1 << 20, file);
                        plan.render_slice(index, &mut |text| {
                            if let Some(key) = key.as_mut() {
                                *key = asm_cache::extend_slice_key(*key, text.as_bytes());
                            }
                            std::io::Write::write_all(&mut writer, text.as_bytes())
                        })?;
                        std::io::Write::flush(&mut writer)?;
                        drop(writer);
                        // A restored object is a HARDLINK to the cache entry, so anything that
                        // writes to it in place would write through into the cache; `reuse`
                        // unlinks the name first, which leaves the entry's inode untouched.
                        if let Some(key) = key {
                            if asm_cache::reuse(key, &slice_obj(index)) {
                                let _ = std::fs::remove_file(&asm);
                                return Ok(SliceOutcome { key: Some(key), assembled: false });
                            }
                        }
                        assemble(target, &asm, &slice_obj(index));
                        Ok(SliceOutcome { key, assembled: true })
                    })
                })
                .collect();
            workers.into_iter().map(|worker| worker.join()).collect()
        });
    let summary = plan.summary;
    drop(mapped);

    let mut keys: Vec<u64> = Vec::with_capacity(slice_count);
    let mut misses: Vec<usize> = Vec::new();
    let mut failed_write = false;
    for (index, outcome) in outcomes.into_iter().enumerate() {
        match outcome {
            Ok(Ok(outcome)) => {
                keys.extend(outcome.key);
                if outcome.assembled {
                    misses.push(index);
                }
            }
            Ok(Err(_)) => failed_write = true,
            Err(_) => {
                // A worker that failed the tool call already exited the process; a
                // panicking one has to be reported here rather than linked around.
                eprintln!("Assembler: a parallel slice panicked");
                process::exit(1);
            }
        }
    }
    if failed_write {
        // A slice that could not be written leaves nothing to link; assemble the file whole.
        for index in 0..slice_count {
            let _ = std::fs::remove_file(slice_asm(index));
            let _ = std::fs::remove_file(slice_obj(index));
        }
        assemble(target, asm_path, obj_path);
        return single();
    }
    let cached = slice_count - misses.len();

    // Publish only what this build actually assembled. A hit was already published
    // by whoever produced it, and re-publishing it would only move its mtime.
    if caching {
        for &index in &misses {
            asm_cache::publish(keys[index], &slice_obj(index));
        }
        asm_cache::prune(&keys);
    }

    // The slice sources are derived files: the `.s` the developer asked for is
    // still on disk untouched. A failing `as` exits before this point, so a
    // slice that could not be assembled is always left behind for inspection.
    let objects: Vec<PathBuf> = (0..slice_count).map(|index| slice_obj(index)).collect();
    for &index in &misses {
        let _ = std::fs::remove_file(slice_asm(index));
    }
    let note = format!(
        "Assembler: {} parallel slices ({} temporaries promoted, {} locals published, {} reused from cache)",
        objects.len(),
        summary.promoted.len(),
        summary.published,
        cached
    );
    AssembledObjects {
        objects,
        note: Some(note),
    }
}

/// Whether slices may carry pre-encoded instructions. `ELEPHC_ASM_ENCODE=0` hands `as` the
/// generated text unchanged, which is also what the object-identity check compares against.
fn encode_enabled() -> bool {
    !matches!(std::env::var("ELEPHC_ASM_ENCODE").as_deref(), Ok("0") | Ok("off"))
}

/// What one slice worker did.
struct SliceOutcome {
    /// The slice's cache key, when the cache is on.
    key: Option<u64>,
    /// Whether the slice went through `as` (a miss) rather than being restored from the cache.
    assembled: bool,
}

/// Packs the compiled objects into a static library with `ar`.
///
/// This deliberately does not go through the link plan. An archive is not a
/// link: bridge staticlibs and managed native packages stay separate `.a` files
/// for the consuming project to link alongside this one, exactly as a C library
/// would leave its own dependencies to its consumer. Rolling them in would
/// duplicate every symbol the host also links directly.
///
/// `rcs` replaces the archive, creates it when absent, and writes the symbol
/// index in one step -- no separate `ranlib` pass is needed on either platform.
pub(crate) fn archive(archive_path: &Path, object: &Path, runtime_object: &Path) {
    // A stale archive would otherwise keep members from the previous build,
    // because `r` replaces matching names but never removes vanished ones.
    let _ = std::fs::remove_file(archive_path);

    let mut ar = Command::new("ar");
    ar.arg("rcs").arg(archive_path).arg(object).arg(runtime_object);
    command::run_tool("ar", &mut ar);
}

/// Removes the symbol table from a linked executable.
///
/// Measured on this compiler's own output: 24 % of a `<?php echo 1;` binary and 28 % of a
/// realistic one, and the share grows with the program rather than shrinking — the symbol table
/// is roughly proportional to the number of declarations while `__text` is not. Nothing in a
/// compiled program reads its own symbols: `Throwable::getTrace()` is unimplemented and the
/// uncaught-exception report prints no stack trace, so the names are dead weight at run time.
///
/// NEVER strips a cdylib. Its exported symbols are its interface, and a host resolving one by
/// `dlsym` would get a null it may well read as "feature absent" rather than as an error.
///
/// Failure is not fatal: a missing or foreign `strip` leaves a larger binary, which is a worse
/// outcome than a failed build for something that is purely a size optimisation.
pub(crate) fn strip_symbols(target: Target, emit: Emit, bin_path: &Path) -> Result<(), String> {
    if emit != Emit::Executable {
        return Ok(());
    }
    let tool = target.strip_cmd();
    match Command::new(tool).arg(bin_path).status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("{tool} exited with {status}")),
        Err(error) => Err(format!("could not run {tool}: {error}")),
    }
}

/// Bakes macOS debug maps into a dSYM before temporary objects are removed.
pub(crate) fn bake_debug_info(target: Target, bin_path: &Path) -> bool {
    if target.platform != Platform::MacOS {
        return true;
    }
    let status = Command::new("dsymutil").arg(bin_path).status();
    matches!(status, Ok(status) if status.success())
}

/// Adapts the existing raw linker arguments into a typed plan and links the output.
///
/// Raw libraries retain their legacy dynamic behavior. New managed dependency
/// integration should call [`link_with_plan`] with exact archive items instead.
#[allow(dead_code)] // Retained as a compatibility adapter while non-pipeline callers migrate.
pub(crate) fn link(
    target: Target,
    emit: Emit,
    bin_path: &Path,
    obj_path: &Path,
    runtime_object_path: &Path,
    extra_link_libs: &[String],
    extra_link_paths: &[String],
    extra_frameworks: &[String],
    forced_whole_archive: &[String],
) {
    let mut plan = LinkPlan::new();
    for library in extra_link_libs {
        plan.push(LinkItem::named_user(library));
    }
    for path in extra_link_paths {
        plan.push(LinkItem::SearchPath(PathBuf::from(path)));
    }
    for framework in extra_frameworks {
        plan.push(LinkItem::Framework(framework.clone()));
    }
    if let Err(error) = link_with_plan(
        target,
        emit,
        bin_path,
        obj_path,
        &[],
        runtime_object_path,
        &plan,
        forced_whole_archive,
    ) {
        eprintln!("Linker error: {error}");
        process::exit(1);
    }
}

/// Resolves bridge inputs and executes a linker command from an already typed plan.
///
/// `extra_objects` carries the second and later slices of a parallel-assembled
/// user object. They are appended immediately after `obj_path`, in slice order,
/// which is the emission order: `ld` concatenates section contents in input
/// order, and an emitted `__data` record's meaning depends on its neighbours.
pub(crate) fn link_with_plan(
    target: Target,
    emit: Emit,
    bin_path: &Path,
    obj_path: &Path,
    extra_objects: &[PathBuf],
    runtime_object_path: &Path,
    plan: &LinkPlan,
    forced_whole_archive: &[String],
) -> Result<(), LinkError> {
    let resolved = bridges::resolve(plan, forced_whole_archive, target.platform)?;
    let prepared = (target.platform == Platform::MacOS)
        .then(|| archive_dedup::prepare(&resolved.plan));
    let render_plan = prepared
        .as_ref()
        .map(|prepared| &prepared.plan)
        .unwrap_or(&resolved.plan);

    // The SDK is selected by the target's Apple variant, not by the host: an
    // iOS build resolves the iOS SDK even though it runs on macOS.
    let apple_sdk = target.apple_sdk_name();
    let sdk_path = (target.platform == Platform::MacOS).then(|| sdk::macos_sdk_path(apple_sdk));
    let sdk_version =
        (target.platform == Platform::MacOS).then(|| sdk::macos_sdk_version(apple_sdk));
    let mac_sdk = sdk_path
        .as_deref()
        .zip(sdk_version.as_deref())
        .map(|(path, version)| MacSdk { path, version });
    let homebrew_paths = if target.platform == Platform::MacOS
        && render_plan.needs_default_macos_library_paths()
    {
        sdk::default_macos_library_paths()
    } else {
        Vec::new()
    };

    let rendered = command::render_link_command(
        target,
        emit,
        LinkPaths {
            bin: bin_path,
            object: obj_path,
            extra_objects,
            runtime: runtime_object_path,
        },
        render_plan,
        resolved.needs_libdl,
        mac_sdk,
        &homebrew_paths,
    );
    command::execute_link_command(rendered);

    if let Some(prepared) = prepared {
        prepared.cleanup();
    }
    Ok(())
}
