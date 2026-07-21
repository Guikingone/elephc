//! Purpose:
//! Resolves static Composer autoload mappings and supported SPL registration patterns.
//! Prefixes Composer `autoload.files` and inlines class files discovered by the AOT autoload registry.
//!
//! Called from:
//! - `crate::pipeline::compile()`
//!
//! Key details:
//! - Runtime autoload callbacks cannot run in native binaries; supported rules are interpreted at compile time.
//! - Composer files execute before the entry program while class-triggered files splice before first use.

mod alias;
mod composer_global_functions;
mod index;
mod interpret;
mod polyfill_prune;
mod registry;
mod rule;
mod walk;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub use registry::Registry;
pub use composer_global_functions::scan_composer_global_functions;

use crate::errors::{CompileError, CompileWarning};
use crate::parser::ast::Program;
use crate::parser::ast::Stmt;
use crate::span::Span;

use walk::{collect_declared_fqns, collect_reference_points};

/// Built-in class-like names that exist in every PHP environment (e.g. `Exception`,
/// `stdClass`, `Iterator`). Seeded into the declared FQN set so references to these
/// types are never treated as autoload demands.
const BUILTIN_CLASS_LIKE_NAMES: &[&str] = &[
    "ArrayAccess",
    "AppendIterator",
    "ArrayIterator",
    "ArrayObject",
    "BadFunctionCallException",
    "BadMethodCallException",
    "CachingIterator",
    "CallbackFilterIterator",
    "Countable",
    "DomainException",
    "EmptyIterator",
    "Error",
    "ArithmeticError",
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

/// Walks up from `start` (the entry file's directory) to the nearest ancestor that
/// contains a `composer.json`, returning that directory as the composer project root.
/// Falls back to `start` when no composer.json is found (a non-composer program).
/// PSR-4/classmap targets and the `vendor/` tree are resolved relative to this root, so
/// an entry in a subdirectory (e.g. Symfony's `public/index.php`) still discovers the
/// root autoload map.
pub fn find_composer_project_root(start: &Path) -> PathBuf {
    let mut dir = start;
    loop {
        if dir.join("composer.json").is_file() {
            return dir.to_path_buf();
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => return start.to_path_buf(),
        }
    }
}

/// Run the autoload pass over a fully resolver+name_resolver-processed
/// program. For every canonical class reference that isn't declared in
/// the program, look it up first in the composer.json PSR-4 index and
/// then in the user-registered closure rules; parse the referenced file,
/// run resolver+name_resolver on it, and append. Iterate until stable.
///
/// Returns the expanded program plus any non-fatal warnings (e.g. an
/// `autoload.files` helper that was skipped because it could not be parsed).
pub fn run(
    mut program: Program,
    base_dir: &Path,
    registry: &Registry,
) -> Result<(Program, Vec<CompileWarning>), CompileError> {
    let mut warnings: Vec<CompileWarning> = Vec::new();
    if registry.is_empty() {
        return Ok((program, warnings));
    }
    let mut included: HashSet<PathBuf> = HashSet::new();

    // -- prefix always-included files first --
    // composer.json's `autoload.files` declares files that must always be
    // included. Prefix them in Composer order so their top-level statements
    // execute before the entry program.
    let mut prefix: Program = Vec::new();
    for path in registry.always_included_files() {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if included.insert(canonical.clone()) {
            // `autoload.files` helpers are always-included but often unreferenced by
            // the app. Tolerate an unparseable or unreadable helper by skipping it and
            // recording a warning rather than aborting the whole build, so one
            // unsupported construct in an unused helper cannot kill compilation.
            // Strict include resolution (`false`): a helper's top-level statements run
            // eagerly at startup, so an unresolvable dynamic include must surface as an
            // error that becomes a skip here, not a degraded stub that would fatal at boot.
            match load_autoloaded_file(&canonical, base_dir, false) {
                Ok(stmts) => prefix.extend(stmts),
                Err(e) => warnings.push(CompileWarning::new(
                    Span::dummy(),
                    &format!(
                        "Autoload: skipped autoload.files helper '{}': {}",
                        canonical.display(),
                        e.message
                    ),
                )),
            }
        }
    }
    if !prefix.is_empty() {
        prefix.extend(program);
        program = prefix;
    }

    // Remove PHP polyfill redefinition guards for functions elephc provides. The
    // guarded wrapper bodies are never materialized, so dropping them keeps the
    // classes they delegate to (e.g. the 97 KB `DeepClone` polyfill) out of the
    // reference graph collected below.
    program = polyfill_prune::prune_provided_function_polyfills(program);

    // Decide which optional `autoload.files` helpers (`u`/`b`/`s`, `dump`/`dd`) the program
    // actually calls, so uncalled ones can be pruned (keeping the heavy classes their bodies
    // reference — `UnicodeString`, `ByteString`, `VarDumper` — out of the closure) while called
    // ones are retained. A helper may be called only from a class that the PSR-4 class-reference
    // iteration loads (e.g. `OutputFormatter` calling `b()`/`s()` via `use function`), so the call
    // set must be gathered AFTER that iteration. But running the iteration with the helper bodies
    // present would drag their referenced classes in even for uncalled helpers. The two-phase
    // survey below resolves the ordering: survey with all optional helper bodies stripped, then
    // prune the real program with the surveyed call set, then load the retained helper bodies'
    // classes.

    // Snapshot the set of files already spliced (the `autoload.files` prefix) so the final
    // class-load iteration can re-splice caller classes the survey parsed without re-parsing the
    // prefix.
    let included_after_prefix = included.clone();

    // Survey phase: strip every optional helper guard so none of their bodies' class references
    // enter the survey graph, then iterate class loading to a fixed point. This loads every
    // PSR-4-referenced caller class and exposes the helpers it calls.
    let survey = polyfill_prune::strip_all_optional_helper_guards(program.clone());
    let survey_loaded = load_referenced_classes(survey, base_dir, registry, &mut included)?;
    let called = walk::collect_called_function_names(&survey_loaded);

    // Reset the included set to the prefix snapshot: caller classes parsed during the survey must
    // be re-spliced into the real program (the survey's splices live only in `survey_loaded`), so
    // they must re-parse-and-splice during the final iteration. The prefix files stay included so
    // they are not re-read.
    included = included_after_prefix;

    // Prune the original program (with helper guards intact) using the surveyed call set: a
    // helper named in `called` is retained, an uncalled one is dropped with its body's class
    // references.
    program = polyfill_prune::prune_unused_optional_helpers_with(program, &called);

    // Final class-load iteration: retained helper bodies now reference their classes
    // (`UnicodeString`/`ByteString` for retained `b()`/`s()`), which get loaded here alongside
    // the re-spliced caller classes.
    program = load_referenced_classes(program, base_dir, registry, &mut included)?;

    Ok((program, warnings))
}

/// Iterates PSR-4 class-reference loading to a fixed point: each pass collects class-like
/// references the program makes but does not declare, resolves them through the registry
/// (PSR-4 then user rules), parses and name-resolves the referenced file, and splices its
/// statements before the referencing statement. Stops when a pass adds no new class file.
/// `included` tracks already-loaded files so a class is parsed at most once per call.
fn load_referenced_classes(
    mut program: Program,
    base_dir: &Path,
    registry: &Registry,
    included: &mut HashSet<PathBuf>,
) -> Result<Program, CompileError> {
    const MAX_ITERATIONS: usize = 64;
    for _ in 0..MAX_ITERATIONS {
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
                    // Referenced classes must load or the program is broken: a class the
                    // app actually uses cannot be tolerated-away like an unreferenced
                    // `autoload.files` helper, so a load failure here is a hard error.
                    // Lenient include resolution (`true`): a dynamic include inside a class
                    // method is lazy and may never run, so an unresolvable one degrades to a
                    // runtime-fatal stub instead of failing the whole compile.
                    let loaded = load_autoloaded_file(&canonical, base_dir, true)?;
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
    Ok(program)
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

/// Try the resolution chain in order: composer.json PSR-4 first, then each
/// user-registered closure rule. Returns the first rule that produces a
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

/// Load, parse, and resolve a single autoloaded PHP file, returning its statements.
///
/// `lenient_includes` selects include-resolution strictness for this file. It is `true` only
/// for lazily-referenced class files: such a file's dynamic `include`/`require` typically sits
/// inside a method that may never run for the program being built (e.g. a polyfill that
/// `require`s a data table by a computed path), so an unresolvable runtime-dynamic path is
/// degraded to a runtime-fatal stub rather than failing compilation. It is `false` for
/// always-included `autoload.files` helpers, whose top-level statements execute eagerly at
/// startup: a degraded stub there would fatal immediately, so those keep the strict behavior
/// and an unresolvable include surfaces as an error the caller turns into a tolerant skip.
fn load_autoloaded_file(
    path: &Path,
    base_dir: &Path,
    lenient_includes: bool,
) -> Result<Program, CompileError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        CompileError::new(
            Span::dummy(),
            &format!("Autoload: cannot read '{}': {}", path.display(), e),
        )
    })?;
    let file_label = path.display().to_string();
    let tokens = crate::lexer::tokenize(&content).map_err(|e| e.with_file(file_label.clone()))?;
    let parsed = crate::parser::parse(&tokens).map_err(|e| e.with_file(file_label.clone()))?;
    let parsed = crate::magic_constants::substitute_file_and_scope_constants(parsed, path);
    // Strict-PHP audit of the autoloaded user file on its freshly parsed AST,
    // before resolution can synthesize compiler-internal names into it.
    crate::strict_php::check_file(&parsed, &file_label)?;
    let include_base = path.parent().unwrap_or(base_dir);
    let resolved = if lenient_includes {
        crate::resolver::resolve_lenient_includes(parsed, include_base)?
    } else {
        crate::resolver::resolve(parsed, include_base)?
    };
    let resolved = alias::collect_aliases(resolved);
    let canonicalized: Vec<Stmt> = crate::name_resolver::resolve(resolved)?;
    // name_resolver has already flattened namespace nodes and canonicalized
    // declarations, so we splice the statements directly into the top-level
    // program.
    Ok(canonicalized)
}
