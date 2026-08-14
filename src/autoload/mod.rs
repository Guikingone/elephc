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
mod index;
mod interpret;
mod registry;
mod rule;
mod walk;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub use registry::Registry;

use crate::errors::CompileError;
use crate::parser::ast::Program;
use crate::parser::ast::Stmt;
use crate::span::Span;

use walk::{collect_declared_fqns, collect_reference_points};

/// Physical source paths for declarations introduced by static source loading.
#[derive(Debug, Default)]
pub struct DeclarationSourceFiles {
    pub class_likes: HashMap<String, String>,
    pub functions: HashMap<String, String>,
}

impl DeclarationSourceFiles {
    /// Merges declaration paths from another loaded source set.
    fn extend(&mut self, other: DeclarationSourceFiles) {
        self.class_likes.extend(other.class_likes);
        self.functions.extend(other.functions);
    }
}

/// Built-in class-like names that exist in every PHP environment (e.g. `Exception`,
/// `stdClass`, `Iterator`). Seeded into the declared FQN set so references to these
/// types are never treated as autoload demands.
const BUILTIN_CLASS_LIKE_NAMES: &[&str] = &[
    "ArgumentCountError",
    "ArrayAccess",
    "AppendIterator",
    "ArrayIterator",
    "ArrayObject",
    "AssertionError",
    "BadFunctionCallException",
    "BadMethodCallException",
    "CachingIterator",
    "CallbackFilterIterator",
    "Countable",
    "DivisionByZeroError",
    "DomainException",
    "EmptyIterator",
    "Error",
    "ArithmeticError",
    "UnhandledMatchError",
    "__PHP_Incomplete_Class",
    "Exception",
    "Fiber",
    "FiberError",
    "Generator",
    "InternalIterator",
    "InvalidArgumentException",
    "Iterator",
    "IteratorAggregate",
    "IteratorIterator",
    "JsonException",
    "JsonSerializable",
    "LengthException",
    "LimitIterator",
    "LogicException",
    "MultipleIterator",
    "NoRewindIterator",
    "OutOfBoundsException",
    "OutOfRangeException",
    "OuterIterator",
    "OverflowException",
    "ParentIterator",
    "RangeException",
    "RecursiveArrayIterator",
    "RecursiveCallbackFilterIterator",
    "RecursiveFilterIterator",
    "RecursiveIterator",
    "RecursiveIteratorIterator",
    "RecursiveTreeIterator",
    "ReflectionAttribute",
    "ReflectionClass",
    "ReflectionObject",
    "ReflectionClassConstant",
    "ReflectionEnumBackedCase",
    "ReflectionEnumUnitCase",
    "ReflectionFunction",
    "ReflectionMethod",
    "ReflectionNamedType",
    "ReflectionParameter",
    "ReflectionProperty",
    "ReflectionUnionType",
    "ReflectionIntersectionType",
    "RuntimeException",
    "SeekableIterator",
    "SortDirection",
    "SplDoublyLinkedList",
    "SplFixedArray",
    "SplObserver",
    "SplQueue",
    "SplStack",
    "SplSubject",
    "Stringable",
    "Throwable",
    "Traversable",
    "TypeError",
    "UnderflowException",
    "UnexpectedValueException",
    "ValueError",
    "stdClass",
];

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
            declaration_sources.extend(loaded_sources);
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
        let reference_points = collect_reference_points(&program);
        let mut insertions: Vec<(usize, Program)> = Vec::new();
        for (stmt_idx, fqn) in reference_points {
            if declared.contains(&fqn) {
                continue;
            }
            if let Some(path) = resolve_class(&fqn, registry) {
                let canonical = path.canonicalize().unwrap_or(path);
                if included.insert(canonical.clone()) {
                    let (loaded, loaded_includes, loaded_sources) =
                        load_autoloaded_file(&canonical, base_dir, defines)?;
                    nested_includes.extend(loaded_includes);
                    declaration_sources.extend(loaded_sources);
                    insertions.push((stmt_idx, loaded));
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

/// Lower any top-level literal `class_alias()` calls left after another
/// expansion pass, such as resolver includes or autoloaded files.
pub fn collect_aliases(program: Program) -> Program {
    alias::collect_aliases(program)
}

/// Inserts PHP's built-in class-like names into `declared` so that references
/// to types like `Exception`, `stdClass`, and `Iterator` are never treated as
/// autoload demands. Called at the start of each autoload iteration.
fn seed_builtin_declared_fqns(declared: &mut HashSet<String>) {
    for name in BUILTIN_CLASS_LIKE_NAMES {
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
    let (resolved, nested_includes) = crate::resolver::resolve_collecting_includes_with_defines(
        parsed,
        path.parent().unwrap_or(base_dir),
        defines,
    )?;
    let resolved = alias::collect_aliases(resolved);
    let canonicalized: Vec<Stmt> = crate::name_resolver::resolve(resolved)?;
    let declaration_sources = declaration_source_files(&canonicalized, &file_label);
    // name_resolver has already flattened namespace nodes and canonicalized
    // declarations, so we splice the statements directly into the top-level
    // program.
    Ok((canonicalized, nested_includes, declaration_sources))
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
}
