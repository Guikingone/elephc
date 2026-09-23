//! Purpose:
//! Resolves static namespace mappings and supported SPL registration patterns.
//! Prefixes eagerly loaded sources and inlines class files discovered by the AOT registry.
//!
//! Called from:
//! - `crate::pipeline::compile()`
//!
//! Key details:
//! - Runtime autoload callbacks cannot run in native binaries; supported rules are interpreted at compile time.
//! - Eager files execute before the entry program while class-triggered files splice before first use.
//! - `run_collecting_included` additionally surfaces the canonical path of every file the pass
//!   loaded, which `crate::opcache_prelude` bakes into the OPcache script manifest.

mod alias;
mod composer_files;
mod dynamic_contracts;
mod index;
mod interpret;
mod registry;
mod rule;
mod walk;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub use registry::Registry;

use crate::errors::CompileError;
use crate::parser::ast::{BinOp, Expr, ExprKind, Program, Stmt, StmtKind};
use crate::span::Span;

use walk::{collect_declared_fqns, collect_reference_points};

/// Physical source paths for declarations introduced by static source loading.
#[derive(Debug, Default)]
pub struct DeclarationSourceFiles {
    pub class_likes: HashMap<String, String>,
    pub functions: HashMap<String, String>,
    /// Parsed source inputs, independent of whether PHP has entered their files.
    pub source_units: std::collections::BTreeMap<PathBuf, crate::resolver::SourceUnit>,
    /// Files whose conditional declarations were dropped rather than bound, mapped to the names
    /// they dropped. See `ResolveState::unbound_conditional_declaration_sources`.
    pub unbound_conditional_declaration_sources: HashMap<PathBuf, Vec<String>>,
    /// Names bound through the function-variant mechanism; subtracted from the dropped names.
    pub bound_conditional_declaration_names: HashSet<String>,
}

impl DeclarationSourceFiles {
    /// Merges declaration paths from another loaded source set.
    fn extend(&mut self, other: DeclarationSourceFiles) -> Result<(), CompileError> {
        for unit in other.source_units.into_values() {
            unit.insert_into(&mut self.source_units, Span::dummy())?;
        }
        self.class_likes.extend(other.class_likes);
        self.functions.extend(other.functions);
        self.unbound_conditional_declaration_sources
            .extend(other.unbound_conditional_declaration_sources);
        self.bound_conditional_declaration_names
            .extend(other.bound_conditional_declaration_names);
        Ok(())
    }
}

/// Run the autoload pass over a fully resolver+name_resolver-processed
/// program. For every canonical class reference that isn't declared in
/// the program, look it up first in the static namespace index and then in
/// user-registered closure rules; parse the referenced file,
/// run resolver+name_resolver on it, and append. Iterate until stable.
///
/// This is the loaded-set-discarding wrapper over [`run_collecting_included`], kept for the
/// call sites that do not bake the OPcache script manifest (the `ir_lower` and
/// `tests/codegen/support` harnesses); only `crate::pipeline` takes the longer form.
#[allow(dead_code)] // Consumed by the test harnesses; `crate::pipeline` uses the collecting form.
pub fn run(
    program: Program,
    base_dir: &Path,
    registry: &Registry,
) -> Result<Program, CompileError> {
    run_collecting_included(program, base_dir, registry).map(|(program, _)| program)
}

/// Same as [`run`], but also returns the CANONICAL path of every source file this pass
/// pulled into the program, each exactly once:
/// - manifest-declared eager sources,
/// - every static-mapping or SPL-rule class file resolved by the fixpoint below,
/// - every `include`/`require` target those files themselves pull in (an autoloaded class
///   file that `require`s a helper compiles that helper into the binary too, so it is just
///   as much a cached script).
///
/// The first two come from the pass's own `included` set, which is already canonicalized
/// with `Path::canonicalize` — the SAME normalization `__FILE__` bakes
/// (`crate::magic_constants::file_pass`) — so the paths are directly comparable with
/// `crate::opcache_prelude::ScriptEntry::path`. The third comes from
/// `resolver::resolve_collecting_includes`, canonicalized identically.
///
/// Nested include paths are accumulated SEPARATELY from `included` rather than being
/// folded into it: `included` doubles as the "already autoloaded" guard, and seeding it
/// with include targets would change which files the fixpoint loads. Keeping them apart
/// makes this function's autoload behavior byte-identical to [`run`]'s.
///
/// The vector is SORTED so a build is byte-reproducible.
pub fn run_collecting_included(
    program: Program,
    base_dir: &Path,
    registry: &Registry,
) -> Result<(Program, Vec<PathBuf>), CompileError> {
    run_collecting_included_with_defines(program, base_dir, registry, &HashSet::new())
}

/// Runs autoload expansion while applying conditional symbols to every physical file loaded.
pub fn run_collecting_included_with_defines(
    program: Program,
    base_dir: &Path,
    registry: &Registry,
    defines: &HashSet<String>,
) -> Result<(Program, Vec<PathBuf>), CompileError> {
    run_collecting_included_with_defines_and_sources(program, base_dir, registry, defines)
        .map(|(program, files, _)| (program, files))
}

/// Runs autoload expansion and retains physical source paths for introduced declarations.
pub fn run_collecting_included_with_defines_and_sources(
    mut program: Program,
    base_dir: &Path,
    registry: &Registry,
    defines: &HashSet<String>,
) -> Result<(Program, Vec<PathBuf>, DeclarationSourceFiles), CompileError> {
    if registry.is_empty() {
        return Ok((program, Vec::new(), DeclarationSourceFiles::default()));
    }
    let mut included: HashSet<PathBuf> = HashSet::new();
    let mut nested_includes: HashSet<PathBuf> = HashSet::new();
    let mut declaration_sources = DeclarationSourceFiles::default();
    // -- prefix always-included files first --
    // Eager source manifests declare files that must always be included. Preserve
    // declaration order so their top-level statements execute before the entry program.
    let mut prefix: Program = Vec::new();
    // Accumulated from every eager file that was READ, including the ones declined below. A
    // declined file is not spliced, so reading this off `prefix` afterwards would miss exactly
    // the polyfills whose functions most need the global fallback — measured: with the two
    // changes in place and this read from `prefix`, Symfony's console died on
    // `symfony\component\varexporter\deepclone_to_array()`, a function declared by a declined
    // eager file that the RUNTIME goes on to declare perfectly well.
    let mut eager_globals: HashSet<String> = HashSet::new();
    for path in registry.always_included_files() {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if included.insert(canonical.clone()) {
            let (loaded, loaded_includes, loaded_sources) =
                load_autoloaded_file(&canonical, base_dir, defines)?;
            // TAKE IT WHOLE OR NOT AT ALL. A conditional declaration under this eager file was
            // dropped instead of bound AND nothing in its subtree bound that name, so splicing
            // what is left would publish a file the compiler only half-took: the runtime skips
            // every file the compiler claims, so the function would be declared by nobody and the
            // first call to it would fail with `Call to undefined function`.
            //
            // MEASURED on Symfony `--web`, four builds, two with this rule and two without. The
            // rule is what makes `GET /` render: without it the route dies on
            // `twig\extension\mb_strtoupper` and the assembly is ~1 426 109 37x bytes; with it
            // all eight prod routes answer byte-identically to `php -S` and the assembly is
            // ~1 426 081 0xx. `polyfill-mbstring/bootstrap.php` is the file in question — it
            // delegates to a `bootstrap80.php` of nothing but `if (!function_exists('mb_…'))`
            // declarations, every one of which was dropped while the file was still reported as
            // included. Leaving it out costs only that it runs interpreted, as php does anyway.
            //
            // THE SUBTRACTION IS LOAD-BEARING. A version guard declares the same function in both
            // branches; only the live one is bound, and condemning a file on the dead branch's
            // drop alone would be wrong. `tests/eager_file_polyfill_tests.rs` holds both shapes.
            let bound = &loaded_sources.bound_conditional_declaration_names;
            // TWO ways the compiler can fail to take an eager file WHOLE, and they need different
            // questions because they have different causes.
            //
            // 1. A conditional declaration under it was dropped and bound nowhere. That is
            //    `symfony/polyfill-mbstring`, whose `return require …` IS expanded and whose
            //    `if (!function_exists('mb_…'))` declarations are then stripped out.
            //
            // 2. An include in it was never expanded at all, so the subtree was never read and
            //    reports neither drops nor bindings. That is `symfony/polyfill-deepclone`, whose
            //    `if (\PHP_VERSION_ID >= 80100) { require __DIR__.'/bootstrap81.php'; }` stays a
            //    runtime include — measured `dropped=[] bound=0`, and `deepclone_to_array` absent
            //    from EVERY compile phase while the runtime skipped the parent on its claim, so
            //    the function was declared by nobody.
            //
            // Declining hands the file to the runtime entirely: it is not spliced either, so its
            // top-level effects run exactly once, as php runs them.
            let lost = loaded_sources
                .unbound_conditional_declaration_sources
                .values()
                .flatten()
                .any(|name| !bound.contains(name))
                || contains_unexpanded_include(&loaded);
            eager_globals.extend(eager_global_function_keys(&loaded));
            // The DROPPED names too. A two-hop polyfill (`bootstrap.php` -> `require
            // bootstrap80.php`) has its declarations stripped out of `loaded` before this point,
            // so walking the statements finds nothing — and those are precisely the functions the
            // RUNTIME will declare, which is what the global fallback is for. Measured on
            // `symfony/polyfill-deepclone`: `deepclone_to_array` is dropped, declared at run time,
            // and called bare from `namespace Symfony\Component\VarExporter`.
            eager_globals.extend(
                loaded_sources
                    .unbound_conditional_declaration_sources
                    .values()
                    .flatten()
                    .filter(|name| !name.trim_start_matches('\\').contains('\\'))
                    .map(|name| crate::names::php_symbol_key(name.trim_start_matches('\\'))),
            );
            // `ELEPHC_EAGER_TRACE=1` reports what the compiler actually TOOK from each eager
            // file, which is the only way to tell the three shapes of a half-taken file apart.
            // Reading it: `dropped` are conditional declarations the gate discarded, `bound` are
            // the ones the function-variant mechanism kept, `globals` are the global functions
            // still present at any depth, and `nested` are the includes that were expanded.
            //
            //   mbstring   lost=true  dropped=["mb_…"]              -> declined, runtime owns it
            //   deepclone  lost=false dropped=[] bound=0 globals={} nested=["bootstrap81.php"]
            //              -> the subtree WAS read and contributed nothing, and nothing was
            //                 reported as dropped either. That combination is not explained by
            //                 any of the three known shapes and is why `deepclone_to_array()` is
            //                 declared by nobody.
            if std::env::var("ELEPHC_EAGER_TRACE").is_ok() {
                eprintln!(
                    "[elephc-eager] file={} lost={lost} dropped={:?} bound={} globals={:?} nested={:?}",
                    canonical.display(),
                    loaded_sources
                        .unbound_conditional_declaration_sources
                        .values()
                        .flatten()
                        .collect::<Vec<_>>(),
                    loaded_sources.bound_conditional_declaration_names.len(),
                    eager_global_function_keys(&loaded),
                    loaded_includes
                        .iter()
                        .map(|path| path
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_default())
                        .collect::<Vec<_>>(),
                );
            }
            if lost {
                included.remove(&canonical);
                continue;
            }
            nested_includes.extend(loaded_includes);
            declaration_sources.extend(loaded_sources)?;
            prefix.extend(loaded);
        }
    }
    // Published from the statements as READ, not from `declaration_sources.functions`: a
    // polyfill's declaration sits inside `if (!function_exists(…))`, and the declaration-source
    // walk records only file-scope ones, so that map is empty for exactly this shape.
    crate::eager_globals::set(eager_globals);
    if !prefix.is_empty() {
        prefix.extend(program);
        program = prefix;
    }
    // Every GLOBAL function the eager files just declared, published before a single class file is
    // resolved. That ordering is what makes PHP's global fallback expressible here: a class file
    // resolved later calls `trigger_deprecation()` bare from inside its own namespace, and the
    // resolver can now answer it from a DECLARATION instead of from a hand-maintained allow-list
    // (`name_resolver::canonical_compat_prelude_function_name`, which has been caught missing a
    // name three times). A namespaced declaration is excluded: PHP falls back to the global
    // namespace only, never to another one.

    loop {
        let mut declared = collect_declared_fqns(&program);
        seed_builtin_declared_fqns(&mut declared);
        let mut reference_points = collect_reference_points(&program);
        reference_points.extend(
            dynamic_contracts::candidates(&program)
                .into_iter()
                .map(|name| (0, name)),
        );
        let mut insertions: Vec<(usize, Program)> = Vec::new();
        for (stmt_idx, fqn) in reference_points {
            if declared.contains(&fqn) {
                continue;
            }
            if let Some(path) = resolve_class(&fqn, registry) {
                let canonical = path.canonicalize().unwrap_or(path);
                if included.insert(canonical.clone()) {
                    let bundle = load_autoloaded_bundle(
                        &fqn,
                        &canonical,
                        base_dir,
                        defines,
                        registry,
                        &declared,
                        &mut included,
                        &mut nested_includes,
                        &mut declaration_sources,
                        0,
                    )?;
                    if !bundle.is_empty() {
                        insertions.push((stmt_idx, bundle));
                    }
                }
            }
        }
        if insertions.is_empty() {
            break;
        }
        let mut offset = 0usize;
        for (stmt_idx, loaded) in insertions {
            let insert_at = stmt_idx + offset;
            offset += loaded.len();
            program.splice(insert_at..insert_at, loaded);
        }
    }

    included.extend(nested_includes);
    let mut loaded_files: Vec<PathBuf> = included.into_iter().collect();
    loaded_files.sort();
    Ok((program, loaded_files, declaration_sources))
}





/// Returns whether any include in these statements is still a RUNTIME include.
///
/// The resolver expands what it can reach statically. What it leaves behind is a file the
/// compiled program will not have read, so an eager entry holding one has not been taken whole —
/// see the call site for the measurement.
fn contains_unexpanded_include(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| {
        matches!(stmt.kind, StmtKind::Include { .. })
            || eager_nested_bodies(stmt)
                .into_iter()
                .any(contains_unexpanded_include)
    })
}

/// Collects the GLOBAL functions an eager file declares, at any statement depth.
///
/// Depth matters: every polyfill writes `if (!function_exists('f')) { function f() {…} }`, so a
/// file-scope-only walk finds nothing. A namespaced declaration is skipped — PHP's bare-call
/// fallback reaches the global namespace and no other.
fn eager_global_function_keys(stmts: &[Stmt]) -> HashSet<String> {
    fn walk(stmts: &[Stmt], out: &mut HashSet<String>) {
        for stmt in stmts {
            if let StmtKind::FunctionDecl { name, .. } = &stmt.kind {
                let bare = name.trim_start_matches('\\');
                if !bare.contains('\\') {
                    out.insert(crate::names::php_symbol_key(bare));
                }
            }
            for nested in eager_nested_bodies(stmt) {
                walk(nested, out);
            }
        }
    }
    let mut out = HashSet::new();
    walk(stmts, &mut out);
    out
}

/// Returns the statement lists nested inside one statement, for [`eager_global_function_keys`].
fn eager_nested_bodies(stmt: &Stmt) -> Vec<&[Stmt]> {
    let mut bodies: Vec<&[Stmt]> = Vec::new();
    match &stmt.kind {
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body,
            ..
        } => {
            bodies.push(then_body);
            bodies.extend(elseif_clauses.iter().map(|(_, body)| body.as_slice()));
            if let Some(body) = else_body {
                bodies.push(body);
            }
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            bodies.push(then_body);
            if let Some(body) = else_body {
                bodies.push(body);
            }
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::Foreach { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::Synthetic(body) => bodies.push(body),
        StmtKind::Switch { cases, default, .. } => {
            bodies.extend(cases.iter().map(|(_, body)| body.as_slice()));
            if let Some(body) = default {
                bodies.push(body);
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            bodies.push(try_body);
            bodies.extend(catches.iter().map(|catch| catch.body.as_slice()));
            if let Some(body) = finally_body {
                bodies.push(body);
            }
        }
        _ => {}
    }
    bodies
}

/// Returns the class-like names the program only ever hands to an existence probe.
///
/// See [`walk::probe_only_class_names`]: these are the classes a closed-world build carries
/// solely so `class_exists()` can answer, and which `class_exists($name, false)` must therefore
/// still report as NOT LOADED.
pub fn probe_only_class_names(program: &Program) -> std::collections::HashSet<String> {
    walk::probe_only_class_names(program)
}

/// Records what the AUTOLOAD PASS already did, so the runtime does not try to do it again.
///
/// Two facts, both derived from the same pass and both needed by the generated program:
///
/// - `preincluded_sources`: the compiler opened these files and spliced their declarations in,
///   which is the inclusion php's autoloader would have performed. An `include_once` reaching
///   one of them at runtime must answer "already included" instead of redeclaring everything.
/// - `deferred_class_loads`: the classes among them that the program only ever hands to an
///   existence probe. php would never have loaded those, so `class_exists($n, false)` has to
///   keep reporting them as not loaded until a probe with autoloading asks for one.
///
/// Every compile path has to call this -- the CLI pipeline and the test harness alike -- or the
/// two behave differently on the same program.
pub fn record_compile_time_inclusions(
    module: &mut crate::ir::Module,
    program: &Program,
    autoloaded_files: &[std::path::PathBuf],
) {
    module.preincluded_sources = autoloaded_files
        .iter()
        .map(|path| path.canonicalize().unwrap_or_else(|_| path.clone()))
        .collect();
    let probe_only: std::collections::HashSet<String> = probe_only_class_names(program)
        .into_iter()
        .map(|name| name.to_ascii_lowercase())
        .collect();
    if probe_only.is_empty() {
        return;
    }
    let autoloaded: std::collections::HashSet<std::path::PathBuf> =
        module.preincluded_sources.iter().cloned().collect();
    module.deferred_class_loads = module
        .declared_class_source_files
        .iter()
        .filter(|(name, file)| {
            // Both sides are canonicalized: a declaration path can arrive in the `/var/...`
            // spelling while the autoload pass reports `/private/var/...` for the same file,
            // and comparing them raw silently matches nothing.
            let declared = std::path::PathBuf::from(file);
            let declared = declared.canonicalize().unwrap_or(declared);
            probe_only.contains(&name.trim_start_matches('\\').to_ascii_lowercase())
                && autoloaded.contains(&declared)
        })
        .map(|(name, _)| name.trim_start_matches('\\').to_ascii_lowercase())
        .collect();
}

/// Lower any top-level literal `class_alias()` calls left after another
/// expansion pass, such as resolver includes or autoloaded files.
pub fn collect_aliases(program: Program) -> Program {
    alias::collect_aliases(program)
}

/// Collects static class-string aliases after name resolution and autoload expansion.
pub(crate) fn collect_resolved_aliases(program: Program) -> Program {
    alias::collect_resolved_aliases(program)
}

/// Returns the canonical class pair when alias arguments are statically resolvable.
pub(crate) fn resolved_class_alias_args(args: &[crate::parser::ast::Expr]) -> Option<(String, String)> {
    alias::resolved_class_alias_args(args)
}

/// Inserts PHP's built-in class-like names into `declared` so that references
/// to types like `Exception`, `stdClass`, and `Iterator` are never treated as
/// autoload demands. Called at the start of each autoload iteration.
fn seed_builtin_declared_fqns(declared: &mut HashSet<String>) {
    // Every catalogued builtin class-like (`Exception`, `stdClass`, `Iterator`, `PDO`, ...)
    // is seeded into the declared FQN set so references to it are never autoload demands.
    for name in crate::types::builtin_classes::builtin_class_like_names() {
        declared.insert((*name).to_string());
    }
}

/// Tries the resolution chain in order: static namespace mappings first, then
/// each user-registered closure rule. Returns the first rule that produces a
/// path matching an existing file on disk.
fn resolve_class(fqn: &str, registry: &Registry) -> Option<PathBuf> {
    if let Some(path) = registry.psr4().lookup(fqn) {
        return Some(path.to_path_buf());
    }
    for rule in registry.rules() {
        if let Some(path) = interpret::resolve(rule, fqn) {
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

/// Returns whether the source selected for an autoload demand can bind that class-like symbol.
///
/// PHP loads a candidate file before it binds the requested declaration. A class whose direct
/// parent, interface, or used trait cannot itself be found therefore remains absent; code guarded
/// by an existence probe can continue, while an actually reached construction still fails through
/// the compiler's normal absent-class path. Keeping such a declaration out of the closed world also
/// prevents an eagerly inspected but dormant file from turning a runtime-conditional failure into a
/// whole-program schema error.
fn autoload_target_can_bind(
    program: &Program,
    target: &str,
    declared: &HashSet<String>,
    registry: &Registry,
) -> bool {
    let Some(dependencies) = direct_binding_dependencies(program, target) else {
        return true;
    };
    let local = collect_declared_fqns(program)
        .into_iter()
        .map(|name| crate::names::php_symbol_key(&name))
        .collect::<HashSet<_>>();
    let available = declared
        .iter()
        .map(|name| crate::names::php_symbol_key(name))
        .collect::<HashSet<_>>();

    dependencies.into_iter().all(|dependency| {
        let key = crate::names::php_symbol_key(&dependency);
        local.contains(&key)
            || available.contains(&key)
            || resolve_class(&dependency, registry).is_some()
    })
}

/// Finds the direct class-like dependencies of the requested declaration in one loaded source.
fn direct_binding_dependencies(program: &Program, target: &str) -> Option<Vec<String>> {
    let target_key = crate::names::php_symbol_key(target.trim_start_matches('\\'));
    for stmt in program {
        match &stmt.kind {
            StmtKind::ClassDecl {
                name,
                extends,
                implements,
                trait_uses,
                ..
            } if crate::names::php_symbol_key(name.trim_start_matches('\\')) == target_key => {
                let mut dependencies = Vec::new();
                if let Some(parent) = extends {
                    dependencies.push(parent.as_canonical().trim_start_matches('\\').to_string());
                }
                dependencies.extend(
                    implements
                        .iter()
                        .map(|name| name.as_canonical().trim_start_matches('\\').to_string()),
                );
                dependencies.extend(trait_uses.iter().flat_map(|trait_use| {
                    trait_use.trait_names.iter().map(|name| {
                        name.as_canonical().trim_start_matches('\\').to_string()
                    })
                }));
                return Some(dependencies);
            }
            StmtKind::InterfaceDecl { name, extends, .. }
                if crate::names::php_symbol_key(name.trim_start_matches('\\')) == target_key =>
            {
                return Some(
                    extends
                        .iter()
                        .map(|name| name.as_canonical().trim_start_matches('\\').to_string())
                        .collect(),
                );
            }
            StmtKind::TraitDecl {
                name, trait_uses, ..
            } if crate::names::php_symbol_key(name.trim_start_matches('\\')) == target_key => {
                return Some(
                    trait_uses
                        .iter()
                        .flat_map(|trait_use| {
                            trait_use.trait_names.iter().map(|name| {
                                name.as_canonical().trim_start_matches('\\').to_string()
                            })
                        })
                        .collect(),
                );
            }
            StmtKind::EnumDecl {
                name, implements, ..
            } if crate::names::php_symbol_key(name.trim_start_matches('\\')) == target_key => {
                return Some(
                    implements
                        .iter()
                        .map(|name| name.as_canonical().trim_start_matches('\\').to_string())
                        .collect(),
                );
            }
            StmtKind::NamespaceBlock { body, .. }
            | StmtKind::Synthetic(body)
            | StmtKind::IncludeOnceGuard { body, .. } => {
                if let Some(dependencies) = direct_binding_dependencies(body, target) {
                    return Some(dependencies);
                }
            }
            _ => {}
        }
    }
    None
}

/// How deep one autoload demand may chase its own dependencies before the outer fixpoint loop
/// takes over. Real inheritance chains are a dozen links at most; the cap only stops a pathological
/// or cyclic graph from overflowing the stack.
const AUTOLOAD_BUNDLE_MAX_DEPTH: usize = 64;

/// Loads one autoloaded file together with the files its own file-scope execution demands,
/// ordered dependency-first.
///
/// PHP's autoloader is DEPTH-FIRST: asking for `AsciiSlugger` runs the loader again for
/// `LocaleAwareInterface` before the class binds, and again for whatever that interface needs.
/// Appending each discovered file at its own first reference point instead is breadth-first, and
/// it can place a dependency AFTER the file that demanded it. `symfony/string`'s `AsciiSlugger.php`
/// guards its whole body with `if (!interface_exists(LocaleAwareInterface::class)) { throw ... }`;
/// the interface landed nine statements too late and the compiled program threw
/// "the symfony/translation-contracts package is not installed" before the entry file's first
/// statement ran.
///
/// Returns an empty program when the target cannot bind here, which is the caller's signal to
/// insert nothing — the path stays in `included` either way, exactly as before.
#[allow(clippy::too_many_arguments)]
fn load_autoloaded_bundle(
    fqn: &str,
    canonical: &Path,
    base_dir: &Path,
    defines: &HashSet<String>,
    registry: &Registry,
    declared: &HashSet<String>,
    included: &mut HashSet<PathBuf>,
    nested_includes: &mut HashSet<PathBuf>,
    declaration_sources: &mut DeclarationSourceFiles,
    depth: usize,
) -> Result<Program, CompileError> {
    let (loaded, loaded_includes, loaded_sources) =
        load_autoloaded_file(canonical, base_dir, defines)?;
    if !autoload_target_can_bind(&loaded, fqn, declared, registry) {
        return Ok(Vec::new());
    }

    let mut bundle: Program = Vec::new();
    if depth < AUTOLOAD_BUNDLE_MAX_DEPTH {
        let mut seen: HashSet<String> = HashSet::new();
        for dependency in walk::collect_file_scope_dependencies(&loaded) {
            if declared.contains(&dependency) || !seen.insert(dependency.clone()) {
                continue;
            }
            let Some(path) = resolve_class(&dependency, registry) else {
                continue;
            };
            let dependency_path = path.canonicalize().unwrap_or(path);
            // Inserting before the recursion is what breaks a dependency cycle: the second
            // visit finds the path already claimed and stops.
            if !included.insert(dependency_path.clone()) {
                continue;
            }
            bundle.extend(load_autoloaded_bundle(
                &dependency,
                &dependency_path,
                base_dir,
                defines,
                registry,
                declared,
                included,
                nested_includes,
                declaration_sources,
                depth + 1,
            )?);
        }
    }

    nested_includes.extend(loaded_includes);
    declaration_sources.extend(loaded_sources)?;
    bundle.extend(loaded);
    Ok(bundle)
}

/// Load, parse, and resolve a single autoloaded PHP file, returning its statements plus the
/// canonical paths of every `include`/`require` target the file itself pulled in (surfaced for
/// the OPcache script manifest — see [`run_collecting_included`]).
fn load_autoloaded_file(
    path: &Path,
    base_dir: &Path,
    defines: &HashSet<String>,
) -> Result<(Program, Vec<PathBuf>, DeclarationSourceFiles), CompileError> {
    let content = crate::source::read_physical_source(path).map_err(|e| {
        CompileError::new(
            Span::dummy(),
            &format!("Autoload: cannot read '{}': {}", path.display(), e),
        )
    })?;
    let file_label = path.display().to_string();
    let source_mode = crate::source::SourceMode::from_path(path);
    let tokens = crate::lexer::tokenize_with_mode(&content, source_mode)
        .map_err(|e| e.with_file(file_label.clone()))?;
    let parsed = crate::parser::parse_with_mode(&tokens, source_mode)
        .map_err(|e| e.with_file(file_label.clone()))?;
    let parsed =
        crate::source::finalize_physical_program(parsed, path, source_mode, defines)?;
    let (resolved, nested_includes, included_sources) =
        crate::resolver::resolve_collecting_includes_with_defines_and_sources(
            parsed,
            path.parent().unwrap_or(base_dir),
            defines,
        )?;
    let resolved = alias::collect_aliases(resolved);
    let canonicalized: Vec<Stmt> = crate::name_resolver::resolve(resolved)?;
    // A file-scope `if` the LANGUAGE PROFILE already decides picks which declarations this file
    // contributes, and php resolves it before any of them exist. Folding it here — on the file's
    // own statement list, while it still IS a file — is what lets the `return` in the taken
    // branch reach the boundary restoration below. See
    // [`fold_file_scope_profile_conditions`], whose doc names the one place this must not move to.
    let canonicalized = fold_file_scope_profile_conditions(canonicalized);
    // An autoloaded file is included in STATEMENT position, so its own top-level `return` ends
    // THAT FILE and never the program that included it. These statements are spliced straight
    // into the entry program, so the boundary has to be restored here or the file's `return`
    // becomes the PROGRAM's return. See [`discard_autoloaded_file_return`].
    let canonicalized = discard_autoloaded_file_return(canonicalized);
    let mut declaration_sources = declaration_source_files(&canonicalized, &file_label);
    // Everything this file's own includes declared is attributed to the file that WROTE it, which
    // the walk above cannot know: by now those statements have been spliced in and are
    // indistinguishable from this file's. Per-file attribution therefore overwrites it.
    declaration_sources.class_likes.extend(included_sources.class_likes);
    declaration_sources.functions.extend(included_sources.functions);
    declaration_sources
        .unbound_conditional_declaration_sources
        .extend(included_sources.unbound_conditional_declaration_sources);
    declaration_sources
        .bound_conditional_declaration_names
        .extend(included_sources.bound_conditional_declaration_names);
    for unit in included_sources.source_units.into_values() {
        unit.insert_into(&mut declaration_sources.source_units, Span::dummy())?;
    }
    crate::resolver::SourceUnit {
        canonical_path: path.canonicalize().unwrap_or_else(|_| path.to_path_buf()),
        mode: source_mode,
        source: content.into(),
    }.insert_into(&mut declaration_sources.source_units, Span::dummy())?;
    // name_resolver has already flattened namespace nodes and canonicalized
    // declarations, so we splice the statements directly into the top-level
    // program.
    let canonicalized = activate_interface_declarations(canonicalized, path, &|_| true);
    Ok((canonicalized, nested_includes, declaration_sources))
}

/// Restores PHP's include boundary for a file the autoload pass splices into the program.
///
/// An autoloaded file is always included in STATEMENT position — Composer's generated
/// `autoload_real.php` does `require $file;` for every `autoload.files` entry, and its class
/// loader's `includeFile()` does the same for a class file. PHP's rule for that shape is that a
/// top-level `return` ends the INCLUDED FILE and hands its value back to the (discarded) include
/// expression; it does not return from the program that included it. `crate::resolver`'s
/// `discard_statement_include_return` already restores exactly this for a written-out
/// `require`/`include`, but the autoload pass does not travel that path: it parses the file on
/// its own and splices the name-resolved statements straight into the entry program's top level,
/// where a `Return` node means "return from the program".
///
/// MEASURED, and the reason this exists: with Composer's eager `autoload.files` in the closed
/// world, `vendor/symfony/polyfill-mbstring/bootstrap.php` contributes its last line —
/// `return require __DIR__.'/bootstrap72.php';` — as top-level statement 160 of a 1,848-statement
/// Symfony `--web` program. Everything after it, 91% of the program including the web prelude's
/// whole error-handling surface, sat after a statement that terminates. `optimize::propagate`
/// then truncated the top level there and the build died in the BACKEND with
/// `call to unknown function error_log`, because the checker's function table had kept what the
/// AST had lost. Nothing about that is Symfony-specific: any `autoload.files` entry ending in
/// `return` does it.
///
/// DECLARATIONS AFTER THE `return` ARE KEPT, because php keeps them. An unconditional top-level
/// `function`/`class` in an included file is bound when the file is COMPILED, not when the
/// statement is reached, so
///
/// ```php
/// <?php return 1; function after_return_fn() {}
/// ```
///
/// leaves `function_exists('after_return_fn')` TRUE in the includer (verified on php 8.5.10).
/// Only the unreachable EXECUTABLE tail is dropped.
///
/// SCANNING THE TOP LEVEL ONLY IS COMPLETE HERE, and that is a property of the call site rather
/// than an assumption: `load_autoloaded_file` runs this AFTER `name_resolver::resolve`, which
/// flattens every `NamespaceBlock` into its parent list (`name_resolver::statements::list`) —
/// including the wrappers the resolver puts around this file's OWN includes, whose file-scope
/// returns `crate::resolver`'s `discard_statement_include_return` has already handled.
fn discard_autoloaded_file_return(mut body: Program) -> Program {
    let Some((index, guarded)) = body.iter().enumerate().find_map(|(index, stmt)| {
        match &stmt.kind {
            StmtKind::Return(_) => Some((index, false)),
            _ if is_bare_if_ending_in_return(&stmt.kind) => Some((index, true)),
            _ => None,
        }
    }) else {
        return body;
    };
    if guarded {
        return end_file_under_guard(body, index);
    }
    let span = body[index].span;
    let tail = body.split_off(index + 1);
    // The returned EXPRESSION still runs — `return require 'x.php';` performs the include — so it
    // is kept as a statement whose value is discarded, which is what the include site does with it.
    let returned = std::mem::replace(
        &mut body[index],
        Stmt::new(StmtKind::Synthetic(Vec::new()), span),
    );
    if let StmtKind::Return(Some(value)) = returned.kind {
        body[index] = Stmt::new(StmtKind::ExprStmt(value), span);
    }
    body.extend(
        tail.into_iter()
            .filter(|stmt| is_file_scope_hoisted_declaration(&stmt.kind)),
    );
    body
}

/// Returns whether this file-scope statement is `if (C) { …; return; }` with nothing else in the
/// chain — a GUARDED END OF FILE, and the second shape a file-scope `return` is written in.
///
/// The chain has to be bare (no `elseif`, no `else`) for the rewrite in
/// [`end_file_under_guard`] to be a rewrite rather than a duplication: with one arm the
/// statements after the `if` run exactly when the condition is false, which an `else` says
/// directly. With several arms they run when SEVERAL conditions were false, and saying that
/// without a flag variable means copying them into every arm.
fn is_bare_if_ending_in_return(kind: &StmtKind) -> bool {
    let StmtKind::If {
        then_body,
        elseif_clauses,
        else_body,
        ..
    } = kind
    else {
        return false;
    };
    elseif_clauses.is_empty()
        && else_body.is_none()
        && matches!(then_body.last().map(|stmt| &stmt.kind), Some(StmtKind::Return(_)))
}

/// Restores php's include boundary for `if (C) { …; return; }` followed by more of the file.
///
/// php's rule is the same one [`discard_autoloaded_file_return`] documents — the `return` ends
/// THE FILE — but the statement is not at the top level, so the top-level scan walked past it and
/// the `return` was left to return from the PROGRAM. MEASURED: `symfony/polyfill-mbstring`'s
/// `bootstrap80.php` ends with `if (extension_loaded('mbstring')) { return; }`, and
/// `extension_loaded('mbstring')` is TRUE in a compiled binary, so the second of twelve eager
/// Composer `files` entries returned from the request program. Every route answered `200` with a
/// zero-byte body and no diagnostic of any kind.
///
/// The rewrite is the identity php already gives us:
///
/// ```php
/// if (C) { A; return; }   REST        ===        if (C) { A; } else { REST }
/// ```
///
/// REST runs exactly when the file did not end, which is what `else` says. DECLARATIONS IN REST
/// STAY AT FILE SCOPE rather than moving into the `else`, for the same reason the top-level path
/// keeps them: php binds an unconditional file-scope declaration when the file is COMPILED, so it
/// is bound whether or not the branch ran.
///
/// REST IS PROCESSED BEFORE IT IS NESTED, so a second guarded end-of-file inside it is rewritten
/// while it is still file scope. That ordering is the whole reason this recurses.
fn end_file_under_guard(mut body: Program, index: usize) -> Program {
    let rest = discard_autoloaded_file_return(body.split_off(index + 1));
    let (declarations, executable): (Program, Program) = rest
        .into_iter()
        .partition(|stmt| is_file_scope_hoisted_declaration(&stmt.kind));
    let span = body[index].span;
    let StmtKind::If {
        condition,
        mut then_body,
        ..
    } = std::mem::replace(&mut body[index].kind, StmtKind::Synthetic(Vec::new()))
    else {
        unreachable!("end_file_under_guard is only reached for a bare `if`");
    };
    // The returned EXPRESSION still runs, exactly as on the top-level path.
    if let Some(returned) = then_body.pop() {
        let return_span = returned.span;
        if let StmtKind::Return(Some(value)) = returned.kind {
            then_body.push(Stmt::new(StmtKind::ExprStmt(value), return_span));
        }
    }
    body[index] = Stmt::new(
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses: Vec::new(),
            else_body: (!executable.is_empty()).then_some(executable),
        },
        span,
    );
    body.extend(declarations);
    body
}

/// Resolves a file-scope `if` whose condition the SELECTED LANGUAGE PROFILE already decides,
/// replacing it with the statements of the branch php would take.
///
/// WHY A FILE NEEDS THIS AT ALL. A library that has to work on several PHP versions ships one
/// entry file that picks the implementation for the running one, and the idiom is a file-scope
/// `return` under a version test:
///
/// ```php
/// <?php
/// if (\PHP_VERSION_ID >= 80000) {
///     return require __DIR__.'/bootstrap80.php';
/// }
/// if (!function_exists('thing')) { function thing() { /* pre-8.0 fallback */ } }
/// ```
///
/// php evaluates that condition BEFORE either declaration exists and contributes exactly one of
/// the two bodies. elephc's profile is fixed at compile time, so the condition is a constant here
/// too — but the `return` sits inside the `if`, where [`discard_autoloaded_file_return`]'s
/// top-level scan cannot see it, so the whole file was contributed: the fallback declaration AND
/// the required file's. MEASURED on the Symfony `--web` build: `symfony/polyfill-php85` and
/// `symfony/polyfill-intl-grapheme` each contributed a `grapheme_levenshtein` php never declares,
/// and the assembler refused the program with `symbol '_fn_grapheme_u_levenshtein' is already
/// defined`. Worse and quieter, the `return` in the taken branch stayed EXECUTABLE at the
/// program's top level, so the spliced file returned from the PROGRAM: a compiled entry that
/// eagerly loads such a file printed nothing at all.
///
/// THIS MUST RUN HERE, ON ONE FILE, AND NOT IN `optimize::fold`. By the time the optimizer sees
/// the program every autoloaded file has been spliced into one flat top level, and folding this
/// `if` there turns the file's `return` into the PROGRAM's: `optimize::propagate` stops at the
/// first statement that does not fall through and deletes everything after it. That is the
/// 1,688-of-1,848-statement deletion [`discard_autoloaded_file_return`] documents, and it
/// surfaced four passes away as `call to unknown function error_log`. Folding while the
/// statements are still A FILE is what makes the difference: the `return` is then this file's,
/// and the boundary restoration immediately below degrades it.
///
/// WHAT COUNTS AS DECIDED is deliberately narrow — see [`profile_condition`]. Only the
/// `PHP_*_VERSION*` family is folded, because those are the constants whose values the compiler
/// FIXES for the whole build and no program can change at runtime. `extension_loaded()`,
/// `function_exists()` and `defined()` are left alone here even where the closed world could
/// answer them: each is a call, and a fold that is wrong about one silently deletes a
/// declaration.
///
/// TOP LEVEL ONLY. A `return` at any deeper nesting is inside a function, a loop or a `try`,
/// where it means what it says; only a file-scope one ends the file. The taken branch's own
/// statements BECOME file scope, so they are folded in turn.
fn fold_file_scope_profile_conditions(body: Program) -> Program {
    let mut folded: Program = Vec::with_capacity(body.len());
    for stmt in body {
        let StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } = stmt.kind
        else {
            folded.push(stmt);
            continue;
        };
        match decided_branch(condition, then_body, elseif_clauses, else_body, stmt.span) {
            Ok(taken) => folded.extend(fold_file_scope_profile_conditions(taken)),
            Err(untouched) => folded.push(untouched),
        }
    }
    folded
}

/// Picks the branch of an `if`/`elseif`/`else` chain the profile decides, or hands the statement
/// back unchanged.
///
/// Conditions are evaluated in source order and the FIRST decided-true one wins, which is php's
/// own rule. An undecidable condition stops the walk and returns the whole statement untouched:
/// a later arm cannot be chosen over a guard whose value is unknown, and an earlier arm was
/// already known false, so nothing is lost by leaving the chain to run.
fn decided_branch(
    condition: Expr,
    then_body: Program,
    elseif_clauses: Vec<(Expr, Program)>,
    else_body: Option<Program>,
    span: Span,
) -> Result<Program, Stmt> {
    let rebuild = |condition, then_body, elseif_clauses, else_body| {
        Stmt::new(
            StmtKind::If {
                condition,
                then_body,
                elseif_clauses,
                else_body,
            },
            span,
        )
    };
    match profile_condition(&condition) {
        Some(true) => return Ok(then_body),
        None => return Err(rebuild(condition, then_body, elseif_clauses, else_body)),
        Some(false) => {}
    }
    for (index, (guard, _)) in elseif_clauses.iter().enumerate() {
        match profile_condition(guard) {
            Some(true) => {
                let mut clauses = elseif_clauses;
                return Ok(clauses.swap_remove(index).1);
            }
            None => return Err(rebuild(condition, then_body, elseif_clauses, else_body)),
            Some(false) => {}
        }
    }
    Ok(else_body.unwrap_or_default())
}

/// Evaluates a condition the compile-time PHP profile decides, or `None` when it does not.
///
/// `None` is the answer for everything this cannot prove, including every call and every
/// variable. Short-circuiting is honoured in the direction that cannot lose a side effect: `A &&
/// B` is decided only once `A` is, because `f() && false` still calls `f()`.
fn profile_condition(expr: &Expr) -> Option<bool> {
    match &expr.kind {
        ExprKind::BoolLiteral(value) => Some(*value),
        ExprKind::Not(inner) => profile_condition(inner).map(|value| !value),
        ExprKind::BinaryOp { left, op, right } => match op {
            BinOp::And => match profile_condition(left)? {
                false => Some(false),
                true => profile_condition(right),
            },
            BinOp::Or => match profile_condition(left)? {
                true => Some(true),
                false => profile_condition(right),
            },
            BinOp::Lt
            | BinOp::Gt
            | BinOp::LtEq
            | BinOp::GtEq
            | BinOp::Eq
            | BinOp::NotEq
            | BinOp::StrictEq
            | BinOp::StrictNotEq => {
                let left = profile_int(left)?;
                let right = profile_int(right)?;
                Some(match op {
                    BinOp::Lt => left < right,
                    BinOp::Gt => left > right,
                    BinOp::LtEq => left <= right,
                    BinOp::GtEq => left >= right,
                    BinOp::Eq | BinOp::StrictEq => left == right,
                    _ => left != right,
                })
            }
            _ => None,
        },
        _ => profile_int(expr).map(|value| value != 0),
    }
}

/// Evaluates an integer expression the compile-time PHP profile decides.
fn profile_int(expr: &Expr) -> Option<i64> {
    match &expr.kind {
        ExprKind::IntLiteral(value) => Some(*value),
        ExprKind::Negate(inner) => profile_int(inner)?.checked_neg(),
        ExprKind::ConstRef(name) => profile_constant(name),
        _ => None,
    }
}

/// Returns the value of a PHP constant the compiler fixes for the whole build.
///
/// The set is the version family and nothing else. Every other compile-time-known constant is
/// left undecided on purpose: this fold DELETES source, so its inputs are limited to the values
/// that cannot differ between this compilation and the program it produces.
///
/// A PHP global constant is case-sensitive and has no namespace, so `\PHP_VERSION_ID` and
/// `PHP_VERSION_ID` are the same constant — the last segment is what is compared, as
/// `crate::opcache_prelude`'s own constant detection already does.
fn profile_constant(name: &crate::names::Name) -> Option<i64> {
    let profile = crate::codegen_support::compile_php_version();
    match name.parts.last()?.as_str() {
        "PHP_VERSION_ID" => Some(i64::from(profile.version_id())),
        "PHP_MAJOR_VERSION" => Some(i64::from(profile.major())),
        "PHP_MINOR_VERSION" => Some(i64::from(profile.minor())),
        "PHP_RELEASE_VERSION" => Some(i64::from(profile.release())),
        _ => None,
    }
}

/// Returns whether php binds this declaration when the file is COMPILED rather than when the
/// statement is reached, which is what makes it survive a `return` earlier in the same file.
///
/// The list mirrors `crate::resolver`'s `is_discoverable_declaration`, which is the same question
/// asked at the other include site; the two must agree or one path keeps a declaration the other
/// drops.
fn is_file_scope_hoisted_declaration(kind: &StmtKind) -> bool {
    matches!(
        kind,
        StmtKind::FunctionDecl { .. }
            | StmtKind::ClassDecl { .. }
            | StmtKind::EnumDecl { .. }
            | StmtKind::InterfaceDecl { .. }
            | StmtKind::TraitDecl { .. }
            | StmtKind::PackedClassDecl { .. }
            | StmtKind::ExternFunctionDecl { .. }
            | StmtKind::ExternClassDecl { .. }
            | StmtKind::ExternGlobalDecl { .. }
    )
}

/// Adds the activation events the ENTRY program's own interface declarations need.
///
/// The resolver strips an `include`d file's interfaces into activation events and the autoload pass
/// does the same for the files it splices, but the entry file's own declarations travel neither
/// path: they reach EIR as declarations, lower to a no-op, and leave the overlay cell that
/// `interface_exists()` reads at zero. `interface_exists()` therefore answered FALSE for an
/// interface declared in the very program being compiled, before and after its declaration.
///
/// Call this after `name_resolver::resolve` (namespaces flattened, every included file's
/// declaration already replaced) and before [`run_collecting_included_with_defines_and_sources`],
/// where the remaining top-level interface declarations are exactly the entry file's own.
pub fn activate_entry_interface_declarations(program: Program, entry_path: &Path) -> Program {
    // An INCLUDED file's declarations are hoisted to the entry program's top level while its
    // activation event stays behind at the include site, which may be nested, conditional, or
    // never reached. From here those declarations look exactly like the entry file's own, and the
    // only thing that tells them apart is the event they already carry. A second event
    // re-declares the interface at runtime -- `Cannot redeclare interface
    // Symfony\\...\\KernelInterface` on a front controller that `require_once`s a vendor interface
    // file -- and one added for a file the program never enters makes `interface_exists()` answer
    // true for an interface PHP never declared.
    let mut activated: HashSet<String> = HashSet::new();
    collect_activated_class_like_names(&program, &mut activated);
    activate_interface_declarations(program, entry_path, &|name| {
        !activated.contains(&crate::names::php_symbol_key(name))
    })
}

/// Collects every class-like name that already carries an activation event, at any depth.
///
/// The event can sit inside an include-once guard, a conditional, a loop, a function body, or a
/// method, so a top-level scan is not enough.
fn collect_activated_class_like_names(program: &[Stmt], out: &mut HashSet<String>) {
    for statement in program {
        match &statement.kind {
            StmtKind::ClassLikeActivate { name, .. } => {
                out.insert(crate::names::php_symbol_key(name.trim_start_matches('\\')));
            }
            StmtKind::NamespaceBlock { body, .. }
            | StmtKind::Synthetic(body)
            | StmtKind::While { body, .. }
            | StmtKind::DoWhile { body, .. }
            | StmtKind::Foreach { body, .. }
            | StmtKind::IncludeOnceGuard { body, .. }
            | StmtKind::FunctionDecl { body, .. } => {
                collect_activated_class_like_names(body, out)
            }
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                collect_activated_class_like_names(then_body, out);
                for (_, body) in elseif_clauses {
                    collect_activated_class_like_names(body, out);
                }
                if let Some(body) = else_body {
                    collect_activated_class_like_names(body, out);
                }
            }
            StmtKind::IfDef {
                then_body,
                else_body,
                ..
            } => {
                collect_activated_class_like_names(then_body, out);
                if let Some(body) = else_body {
                    collect_activated_class_like_names(body, out);
                }
            }
            StmtKind::For {
                init, update, body, ..
            } => {
                if let Some(init) = init {
                    collect_activated_class_like_names(std::slice::from_ref(init.as_ref()), out);
                }
                if let Some(update) = update {
                    collect_activated_class_like_names(std::slice::from_ref(update.as_ref()), out);
                }
                collect_activated_class_like_names(body, out);
            }
            StmtKind::Switch { cases, default, .. } => {
                for case in cases {
                    collect_activated_class_like_names(&case.1, out);
                }
                if let Some(body) = default {
                    collect_activated_class_like_names(body, out);
                }
            }
            StmtKind::Try {
                try_body,
                catches,
                finally_body,
            } => {
                collect_activated_class_like_names(try_body, out);
                for catch in catches {
                    collect_activated_class_like_names(&catch.body, out);
                }
                if let Some(body) = finally_body {
                    collect_activated_class_like_names(body, out);
                }
            }
            _ => {}
        }
    }
}

/// Adds the activation event every interface this file declares needs to be visible to
/// `interface_exists()`.
///
/// `interface_exists()` answers from a per-request activation cell, and NOTHING BUT a
/// `ClassLikeActivate` event ever writes that cell. `crate::resolver` emits one for each interface
/// it strips out of an `include`d file, but an autoloaded file never travels that path: it kept an
/// overlay cell that no statement could set, so `interface_exists()` answered false for the rest of
/// the program however early the declaration ran. `symfony/string`'s `AsciiSlugger.php` opens with
/// `if (!interface_exists(LocaleAwareInterface::class)) { throw ... }` and threw its
/// "symfony/translation-contracts is not installed" LogicException on boot because of it.
///
/// The declaration statement itself stays: unlike the include path, nothing has extracted it yet.
///
/// PHP early-binds an interface that extends nothing — it exists from the moment its file is
/// entered, ahead of the file's own statements. One that extends another interface binds where it
/// is written.
fn activate_interface_declarations(
    program: Program,
    path: &Path,
    accept: &dyn Fn(&str) -> bool,
) -> Program {
    use crate::parser::ast::ClassLikeKind;

    let mut early: Program = Vec::new();
    let mut rest: Program = Vec::with_capacity(program.len());
    for stmt in program {
        let StmtKind::InterfaceDecl {
            ref name,
            ref extends,
            ..
        } = stmt.kind
        else {
            rest.push(stmt);
            continue;
        };
        if !accept(name.trim_start_matches('\\')) {
            rest.push(stmt);
            continue;
        }
        let event = Stmt::new(
            StmtKind::ClassLikeActivate {
                name: name.clone(),
                kind: ClassLikeKind::Interface,
                source_path: path.to_path_buf(),
            },
            stmt.span,
        );
        if extends.is_empty() {
            early.push(event);
            rest.push(stmt);
        } else {
            rest.push(stmt);
            rest.push(event);
        }
    }
    early.extend(rest);
    early
}

/// Collects canonical declaration names associated with one physical source file.
fn declaration_source_files(program: &Program, source_file: &str) -> DeclarationSourceFiles {
    let mut sources = DeclarationSourceFiles::default();
    collect_declaration_source_files(program, source_file, &mut sources);
    sources
}

/// Recurses through transparent statement wrappers while recording declaration paths.
fn collect_declaration_source_files(
    program: &Program,
    source_file: &str,
    sources: &mut DeclarationSourceFiles,
) {
    for stmt in program {
        match &stmt.kind {
            crate::parser::ast::StmtKind::ClassDecl { name, .. }
            | crate::parser::ast::StmtKind::InterfaceDecl { name, .. }
            | crate::parser::ast::StmtKind::TraitDecl { name, .. }
            | crate::parser::ast::StmtKind::EnumDecl { name, .. }
            | crate::parser::ast::StmtKind::PackedClassDecl { name, .. } => {
                sources
                    .class_likes
                    .insert(
                        crate::names::php_symbol_key(name.trim_start_matches('\\')),
                        source_file.to_string(),
                    );
            }
            crate::parser::ast::StmtKind::FunctionDecl { name, .. } => {
                sources
                    .functions
                    .insert(
                        crate::names::php_symbol_key(name.trim_start_matches('\\')),
                        source_file.to_string(),
                    );
            }
            crate::parser::ast::StmtKind::NamespaceBlock { body, .. }
            | crate::parser::ast::StmtKind::Synthetic(body) => {
                collect_declaration_source_files(body, source_file, sources);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod source_units_tests;

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies loaded declaration paths use PHP's case-insensitive symbol keys.
    #[test]
    fn declaration_source_files_normalize_class_and_function_names() {
        let tokens = crate::lexer::tokenize(
            "<?php namespace Demo; class Thing {} function execute(): void {}",
        )
        .expect("tokenization should succeed");
        let parsed = crate::parser::parse(&tokens).expect("parsing should succeed");
        let resolved = crate::name_resolver::resolve(parsed).expect("resolution should succeed");
        let sources = declaration_source_files(&resolved, "/tmp/Thing.php");
        assert_eq!(
            sources
                .class_likes
                .get(&crate::names::php_symbol_key("DEMO\\THING"))
                .map(String::as_str),
            Some("/tmp/Thing.php")
        );
        assert_eq!(
            sources
                .functions
                .get(&crate::names::php_symbol_key("DEMO\\EXECUTE"))
                .map(String::as_str),
            Some("/tmp/Thing.php")
        );
    }

    /// Verifies declaration binding dependencies autoload while callable signature types stay lazy.
    #[test]
    fn reference_points_defer_named_callable_signature_types() {
        let tokens = crate::lexer::tokenize(
            r#"<?php
namespace Fixtures;
class Carrier {
    private PropertyType $property;
    use PrimaryTrait, SecondaryTrait { PrimaryTrait::run insteadof SecondaryTrait; }
    public function method(MethodParam $param, MethodVariadic ...$rest): MethodReturn {}
}
function free(FunctionParam $param, FunctionVariadic ...$rest): FunctionReturn {}
$closure = function (ClosureParam $param, ClosureVariadic ...$rest): ClosureReturn {};
try {} catch (CaughtOne|CaughtTwo) {}
enum Choice implements EnumContract {
    use EnumTrait;
    public function accept(EnumMethodParam $param): EnumMethodReturn {}
}
"#,
        )
        .expect("tokenization should succeed");
        let parsed = crate::parser::parse(&tokens).expect("parsing should succeed");
        let resolved = crate::name_resolver::resolve(parsed).expect("resolution should succeed");
        let references = collect_reference_points(&resolved)
            .into_iter()
            .map(|(_, name)| name)
            .collect::<HashSet<_>>();
        let expected = [
            "Fixtures\\PrimaryTrait",
            "Fixtures\\SecondaryTrait",
            "Fixtures\\CaughtOne",
            "Fixtures\\CaughtTwo",
            "Fixtures\\EnumContract",
            "Fixtures\\EnumTrait",
        ];
        for name in expected {
            assert!(
                references.contains(name),
                "missing autoload reference {name}; collected {references:?}"
            );
        }
        for name in [
            "Fixtures\\PropertyType",
            "Fixtures\\MethodParam",
            "Fixtures\\MethodVariadic",
            "Fixtures\\MethodReturn",
            "Fixtures\\FunctionParam",
            "Fixtures\\FunctionVariadic",
            "Fixtures\\FunctionReturn",
            "Fixtures\\ClosureParam",
            "Fixtures\\ClosureVariadic",
            "Fixtures\\ClosureReturn",
            "Fixtures\\EnumMethodParam",
            "Fixtures\\EnumMethodReturn",
        ] {
            assert!(
                !references.contains(name),
                "signature type unexpectedly triggered autoload: {name}; collected {references:?}"
            );
        }
    }

    /// Verifies a `[Foo::class, 'method']` callable array is an autoload root, and a bare
    /// `Foo::class` still is not.
    ///
    /// The distinction is the whole point. PHP resolves `Foo::class` lexically and never consults
    /// the autoloader for it — a `sprintf()` argument naming a class from a package the app does
    /// not install must stay ignored, which was measured when collecting every `::class` pulled
    /// `DoctrineDbalAdapter` into a Symfony build off exactly such an argument. A CALLABLE ARRAY
    /// is built to be called, and calling it is what runs the loader.
    ///
    /// Twig's `EscaperExtension` is the case: `[FileExtensionEscapingStrategy::class, 'guess']` was
    /// the program's only reference to that class, and the build reported `Undefined class` for a
    /// file sitting in the same package.
    #[test]
    fn reference_points_include_callable_array_class_constants() {
        let tokens = crate::lexer::tokenize(
            r#"<?php
namespace Fixtures;
class Carrier {
    public function build(): array {
        $strategy = [EscapingStrategy::class, 'guess'];
        $pair = [DataOnly::class, 42];
        $message = \sprintf('%s is not enabled', MentionedOnly::class);
        return [$strategy, $pair, $message];
    }
}
"#,
        )
        .expect("tokenization should succeed");
        let parsed = crate::parser::parse(&tokens).expect("parsing should succeed");
        let resolved = crate::name_resolver::resolve(parsed).expect("resolution should succeed");
        let references = collect_reference_points(&resolved)
            .into_iter()
            .map(|(_, name)| name)
            .collect::<HashSet<_>>();
        assert!(
            references.contains("Fixtures\\EscapingStrategy"),
            "a callable array's class must autoload; collected {references:?}"
        );
        for name in ["Fixtures\\DataOnly", "Fixtures\\MentionedOnly"] {
            assert!(
                !references.contains(name),
                "`::class` outside a callable array must not autoload: {name}; collected {references:?}"
            );
        }
    }

    /// Verifies attribute-only class strings become autoload discovery roots, including nesting.
    #[test]
    fn reference_points_include_attribute_class_constant_dependencies() {
        let tokens = crate::lexer::tokenize(
            r#"<?php
namespace Fixtures;
#[Metadata(Dependency::class, nested: [NestedDependency::class])]
class Carrier {}
"#,
        )
        .expect("tokenization should succeed");
        let parsed = crate::parser::parse(&tokens).expect("parsing should succeed");
        let resolved = crate::name_resolver::resolve(parsed).expect("resolution should succeed");
        let references = collect_reference_points(&resolved)
            .into_iter()
            .map(|(_, name)| name)
            .collect::<HashSet<_>>();
        for name in [
            "Fixtures\\Metadata",
            "Fixtures\\Dependency",
            "Fixtures\\NestedDependency",
        ] {
            assert!(
                references.contains(name),
                "missing attribute dependency {name}; collected {references:?}"
            );
        }
    }
}
