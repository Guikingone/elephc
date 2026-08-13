//! Purpose:
//! Collects preload symbols and computes preload verdicts and statistics.
//!
//! Called from:
//! - The OPcache prelude facade and sibling rendering modules.
//!
//! Key details:
//! - Startup failures remain compile errors and manifest membership remains explicit.

#[allow(unused_imports)]
use super::*;

/// The `opcache.preload` directive name.
pub(super) const PRELOAD_DIRECTIVE: &str = "opcache.preload";

/// The user-declared PHP symbol names an elephc binary bakes, in source declaration order.
///
/// Collected by [`collect_preload_symbols`] from the resolved AST BEFORE any compiler prelude is
/// injected, so the lists carry USER functions/classes only — never `var_export`, the PDO surface,
/// or the OPcache functions this module itself injects. That mirrors reference PHP, where
/// `preload_statistics` reports the DELTA the preload pass added to the symbol tables and can
/// therefore never contain a built-in.
#[derive(Debug, Default, Clone)]
pub struct PreloadSymbols {
    /// Fully-qualified function names, original case, no leading `\` (the reference spelling —
    /// VERIFIED: a `namespace My\Space; function MixedCaseFn(){}` preload reports
    /// `My\Space\MixedCaseFn`).
    pub(super) functions: Vec<String>,
    /// Fully-qualified class-like names, original case, no leading `\`. Reference PHP puts
    /// classes, INTERFACES, TRAITS and ENUMS all under the single `classes` key — VERIFIED on
    /// PHP 8.5.6 with one preload file declaring all four.
    pub(super) classes: Vec<String>,
}

/// Collects the user-declared function and class-like names of a resolved program.
///
/// Recursion set is EXACTLY the one `detect::stmt_declares` uses (`NamespaceBlock`,
/// `IncludeOnceGuard`, `Synthetic`): those are the block forms that can host a hoisted top-level
/// declaration. Conditionally-declared functions (inside `if`/loops) are not collected, matching
/// that precedent — and matching reference PHP, where preloading a file whose declarations are
/// conditional does not add them either (the preload pass runs the file, but elephc's manifest
/// analogue is static).
///
/// Names are canonicalized with [`canonical_name_for_decl`] against the enclosing namespace, since
/// this runs BEFORE `name_resolver` and the raw `FunctionDecl`/`ClassDecl` names are local.
/// Duplicates are dropped case-insensitively (PHP symbol names are case-insensitive, and elephc's
/// `FunctionVariantGroup` extension can surface one PHP-visible name through several declarations).
pub fn collect_preload_symbols(program: &[Stmt]) -> PreloadSymbols {
    let mut symbols = PreloadSymbols::default();
    let mut seen_functions: HashSet<String> = HashSet::new();
    let mut seen_classes: HashSet<String> = HashSet::new();
    collect_symbols_in(program, None, &mut symbols, &mut seen_functions, &mut seen_classes);
    symbols
}

/// Walks one statement list under `namespace`, appending every declaration it hosts.
///
/// `NamespaceDecl` (the statement form, `namespace X;`) rebinds the namespace for the REST of the
/// current list; `NamespaceBlock` (the brace form) scopes it to its own body only.
pub(super) fn collect_symbols_in(
    body: &[Stmt],
    namespace: Option<&str>,
    out: &mut PreloadSymbols,
    seen_functions: &mut HashSet<String>,
    seen_classes: &mut HashSet<String>,
) {
    let mut current: Option<String> = namespace.map(str::to_string);
    for stmt in body {
        match &stmt.kind {
            StmtKind::NamespaceDecl { name } => {
                current = name.as_ref().map(Name::as_canonical);
            }
            StmtKind::NamespaceBlock { name, body } => {
                collect_symbols_in(
                    body,
                    name.as_ref().map(Name::as_str),
                    out,
                    seen_functions,
                    seen_classes,
                );
            }
            StmtKind::IncludeOnceGuard { body, .. } | StmtKind::Synthetic(body) => {
                collect_symbols_in(body, current.as_deref(), out, seen_functions, seen_classes);
            }
            StmtKind::FunctionDecl { name, .. } => {
                let fqn = canonical_name_for_decl(current.as_deref(), name);
                if seen_functions.insert(fqn.to_ascii_lowercase()) {
                    out.functions.push(fqn);
                }
            }
            // Reference PHP reports all four class-like kinds under `classes` (VERIFIED).
            // `PackedClassDecl` is elephc's `#[Packed] class` extension — still a user-declared
            // class from PHP's point of view, so it is reported like any other.
            StmtKind::ClassDecl { name, .. }
            | StmtKind::EnumDecl { name, .. }
            | StmtKind::InterfaceDecl { name, .. }
            | StmtKind::TraitDecl { name, .. }
            | StmtKind::PackedClassDecl { name, .. } => {
                let fqn = canonical_name_for_decl(current.as_deref(), name);
                if seen_classes.insert(fqn.to_ascii_lowercase()) {
                    out.classes.push(fqn);
                }
            }
            _ => {}
        }
    }
}

/// The COMPILE-TIME verdict on `opcache.preload` for this binary.
///
/// THE REFERENCE MATRIX, all four rows VERIFIED against reference PHP 8.5.6 (Homebrew, `Zend
/// OPcache` loaded), not derived from php-src:
/// - `opcache.preload` EMPTY (the default) → no preloading, `opcache_get_status()` carries NO
///   `preload_statistics` key. (Also verified with an explicit `-d opcache.preload=`.)
/// - Set + cache ENABLED + the path RESOLVES → `preload_statistics` appears, between
///   `opcache_statistics` and `scripts`.
/// - Set + cache ENABLED + the path does NOT resolve → reference FATALS AT STARTUP, before a
///   single line of the script runs, and exits 1:
///   `PHP Warning:  PHP Startup: Failed to open stream: No such file or directory in Unknown on
///   line 0` then `PHP Fatal error:  Failed opening required '<path>' (include_path='…') in
///   Unknown on line 0`.
/// - Set + cache DISABLED (`opcache.enable_cli=0`, the CLI default) → the process runs FINE,
///   nothing is preloaded (`function_exists()` on a preload-file symbol is `false`) and
///   `opcache_get_status()` returns `false`. A MISSING path in this state is also harmless: no
///   validation happens at all, exit 0.
///
/// WHY A COMPILE ERROR IS THE HONEST AOT MAPPING of the startup fatal: reference resolves
/// `opcache.preload` once, at process startup, before user code. elephc's INI is fixed when the
/// binary is built (`--ini` is a compile-time flag), so "startup" for an elephc binary IS compile
/// time. Refusing to build is the only way to avoid shipping a binary that would report
/// statistics for a file that is not there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreloadVerdict {
    /// `opcache.preload` is empty, OR the OPcache cache is disabled for this target. Nothing is
    /// validated and no `preload_statistics` key is emitted — byte-identical to the behavior
    /// before this feature existed.
    NotPreloading,
    /// Set, cache enabled, but the path does not resolve to a readable file: a COMPILE ERROR.
    /// Carries the directive value VERBATIM (not canonicalized — there is nothing to canonicalize),
    /// which is also the spelling reference PHP puts in its fatal.
    Unresolvable {
        /// The raw `opcache.preload` value as written.
        requested: String,
    },
    /// Set, cache enabled, path resolves. `in_manifest` is whether the resolved path is one of the
    /// scripts elephc actually baked into this binary; `false` earns a compile WARNING but not an
    /// error, because a preload file that this program never includes, requires or autoloads is a
    /// legitimate configuration — reference PHP would preload it, elephc simply does not compile
    /// it in. The membership test must be made against the COMPLETE manifest, so `crate::pipeline`
    /// evaluates the warning only after `autoload::run` (see [`bake_manifest`]).
    Preloading {
        /// The canonicalized preload path (same normalization `__FILE__` and [`ScriptEntry`] use).
        resolved: String,
        /// Whether `resolved` is a member of the compile-time script manifest.
        in_manifest: bool,
    },
}

impl PreloadVerdict {
    /// The compile-error text for the unresolvable case, or `None` when the build may proceed.
    /// Names the directive and the unresolvable path, and states why a startup fatal became a
    /// compile error.
    pub fn compile_error(&self) -> Option<String> {
        match self {
            PreloadVerdict::Unresolvable { requested } => Some(format!(
                "opcache.preload: failed opening required '{requested}': no such readable file. \
                 Reference PHP resolves opcache.preload during startup and FATALS there when the \
                 file is missing; elephc resolves it at compile time, so the equivalent failure is \
                 this compile error. Fix the path, or drop `--ini opcache.preload=…`."
            )),
            _ => None,
        }
    }

    /// The compile-warning text for a resolvable preload file that is NOT in the compile-time
    /// script manifest, or `None` when there is nothing to warn about. Not an error: preloading a
    /// file this program never includes/requires/autoloads is a legitimate configuration that must
    /// not break a build.
    pub fn compile_warning(&self) -> Option<String> {
        match self {
            PreloadVerdict::Preloading {
                resolved,
                in_manifest: false,
            } => Some(format!(
                "opcache.preload: '{resolved}' is not in this binary's compile-time OPcache script \
                 manifest (the entry file, its statically-resolved includes, and its autoloaded \
                 files), so it is not compiled into the binary; \
                 opcache_get_status()['preload_statistics'] reports the manifest and the \
                 symbols this binary actually bakes instead."
            )),
            _ => None,
        }
    }
}

/// Decides — AT COMPILE TIME — what `opcache.preload` means for this binary. See
/// [`PreloadVerdict`] for the verified reference matrix each arm reproduces.
///
/// The cache-enabled test comes SECOND on purpose: with the cache off, reference PHP never looks at
/// the directive at all (verified: a missing preload path with `opcache.enable_cli=0` runs cleanly),
/// so elephc must not validate the path either.
///
/// The path is resolved with `Path::canonicalize` — the same normalization `__FILE__`,
/// [`ScriptEntry::path`] and [`canonical_entry_path`] use — so manifest membership is compared on
/// equal spellings (on macOS a `/tmp/...` preload resolves to `/private/tmp/...`, exactly as
/// reference PHP reports it in `preload_statistics.scripts`). A path that canonicalizes to a
/// DIRECTORY is treated as unresolvable: reference cannot `require` a directory either.
pub fn preload_verdict(
    php_version: PhpVersion,
    web: bool,
    overrides: &[(String, String)],
    manifest: &[ScriptEntry],
) -> PreloadVerdict {
    let version_id = php_version.version_id();
    let requested = directive_str(version_id, PRELOAD_DIRECTIVE, overrides);
    if requested.is_empty() {
        return PreloadVerdict::NotPreloading;
    }
    if !opcache_cache_enabled_with_overrides(version_id, web, overrides) {
        return PreloadVerdict::NotPreloading;
    }
    let Ok(canonical) = Path::new(&requested).canonicalize() else {
        return PreloadVerdict::Unresolvable { requested };
    };
    if !canonical.is_file() {
        return PreloadVerdict::Unresolvable { requested };
    }
    let resolved = canonical.display().to_string();
    let in_manifest = manifest.iter().any(|entry| entry.path == resolved);
    PreloadVerdict::Preloading {
        resolved,
        in_manifest,
    }
}

/// The baked `opcache_get_status()['preload_statistics']` block.
///
/// THE REFERENCE SHAPE, VERIFIED on PHP 8.5.6 (`php -d opcache.enable=1 -d opcache.enable_cli=1
/// -d opcache.preload=<file> -r 'var_export(opcache_get_status());'`) — keys in THIS order:
/// `memory_consumption` (int), `functions` (list<string>), `classes` (list<string>),
/// `scripts` (list<string>).
///
/// CRUCIALLY, `functions` and `classes` are OMITTED ENTIRELY when empty — they are not reported as
/// empty arrays. VERIFIED by preloading a file containing only `<?php`: the block came back as just
/// `['memory_consumption' => 568, 'scripts' => ['…/empty.php']]`. `memory_consumption` and
/// `scripts` are always present. This renderer reproduces that omission, so elephc never emits a
/// shape reference PHP cannot produce.
///
/// TOP-LEVEL SHAPE: preloading adds `preload_statistics` and NOTHING ELSE to the status array —
/// there is no `preload_cached_scripts` key (VERIFIED by diffing the top-level key list with and
/// without preloading: `opcache_enabled, cache_full, restart_pending, restart_in_progress,
/// memory_usage, interned_strings_usage, opcache_statistics, scripts, jit` gains exactly one
/// entry, `preload_statistics`, in eighth position). `preload_statistics` is also NOT suppressed
/// by `opcache_get_status(false)`; only `scripts` is.
///
/// DOCUMENTED DIVERGENCE (verified, deliberately not reproduced): reference PHP additionally
/// inserts a SYNTHETIC `$PRELOAD$` pseudo-entry into the top-level `scripts` map (with
/// `full_path` literally `$PRELOAD$` and `memory_consumption` equal to
/// `preload_statistics.memory_consumption`), which also bumps
/// `opcache_statistics.num_cached_scripts` by one. That entry stands for the shared-memory block
/// preloading itself allocates. An elephc binary allocates no such block — its scripts are native
/// code in the executable — so fabricating a `$PRELOAD$` script would be inventing a cache entry
/// that does not exist. `scripts` and `num_cached_scripts` therefore keep reporting exactly the
/// manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreloadStatistics {
    /// Total memory the preloaded scripts occupy. Derived as Σ of the manifest entries'
    /// `memory_consumption`, so it stays coherent with the `scripts` map and `used_memory` that the
    /// same manifest feeds (implementation-defined in reference PHP too — no two builds agree).
    pub(super) memory_consumption: i64,
    /// Fully-qualified user function names this binary bakes.
    pub(super) functions: Vec<String>,
    /// Fully-qualified user class/interface/trait/enum names this binary bakes.
    pub(super) classes: Vec<String>,
    /// Canonical paths of the preloaded scripts — the compile-time script manifest.
    pub(super) scripts: Vec<String>,
}

/// Builds the `preload_statistics` block from the compile-time manifest and symbol tables, or
/// `None` when [`preload_verdict`] says this binary does not preload.
///
/// WHY THE MANIFEST IS THE RIGHT SOURCE: this repo has already committed to "the AOT binary IS the
/// cache" — `opcache_compile_file` returns `true` for manifest members, and `scripts` /
/// `num_cached_scripts` already report the manifest. Reporting preload statistics from the same
/// source is the consistent choice, and it is the ONLY source that is true: an elephc binary really
/// does hold every one of those scripts and symbols resident for its whole life, which is precisely
/// what preloading buys in reference PHP.
///
/// `functions` / `classes` are REAL SYMBOLS, not an empty-list interim: they come from
/// [`collect_preload_symbols`] walking the resolved pre-prelude AST. They are, however, the symbols
/// of the WHOLE binary rather than of the preload file specifically — an elephc binary cannot
/// separate "preloaded" from "compiled in", because everything it bakes is permanently resident.
/// That is a documented divergence, and it is a superset relationship, never a fabrication: every
/// name reported is genuinely declared by this program.
pub fn preload_statistics(
    verdict: &PreloadVerdict,
    manifest: &[ScriptEntry],
    symbols: &PreloadSymbols,
) -> Option<PreloadStatistics> {
    if !matches!(verdict, PreloadVerdict::Preloading { .. }) {
        return None;
    }
    Some(PreloadStatistics {
        memory_consumption: manifest.iter().map(|entry| entry.memory_consumption).sum(),
        functions: symbols.functions.clone(),
        classes: symbols.classes.clone(),
        scripts: manifest.iter().map(|entry| entry.path.clone()).collect(),
    })
}
