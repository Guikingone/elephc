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
use crate::parser::ast::{Program, Stmt, StmtKind};
use crate::span::Span;

use walk::{collect_declared_fqns, collect_reference_points};

/// Physical source paths for declarations introduced by static source loading.
#[derive(Debug, Default)]
pub struct DeclarationSourceFiles {
    pub class_likes: HashMap<String, String>,
    pub functions: HashMap<String, String>,
    /// Parsed source inputs, independent of whether PHP has entered their files.
    pub source_units: std::collections::BTreeMap<PathBuf, crate::resolver::SourceUnit>,
}

impl DeclarationSourceFiles {
    /// Merges declaration paths from another loaded source set.
    fn extend(&mut self, other: DeclarationSourceFiles) -> Result<(), CompileError> {
        for unit in other.source_units.into_values() {
            unit.insert_into(&mut self.source_units, Span::dummy())?;
        }
        self.class_likes.extend(other.class_likes);
        self.functions.extend(other.functions);
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
    for path in registry.always_included_files() {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if included.insert(canonical.clone()) {
            let (loaded, loaded_includes, loaded_sources) =
                load_autoloaded_file(&canonical, base_dir, defines)?;
            nested_includes.extend(loaded_includes);
            declaration_sources.extend(loaded_sources)?;
            prefix.extend(loaded);
        }
    }
    if !prefix.is_empty() {
        prefix.extend(program);
        program = prefix;
    }

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
    let mut declaration_sources = declaration_source_files(&canonicalized, &file_label);
    // Everything this file's own includes declared is attributed to the file that WROTE it, which
    // the walk above cannot know: by now those statements have been spliced in and are
    // indistinguishable from this file's. Per-file attribution therefore overwrites it.
    declaration_sources.class_likes.extend(included_sources.class_likes);
    declaration_sources.functions.extend(included_sources.functions);
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
