//! Purpose:
//! Coordinates PHP namespace and import resolution across a parsed program.
//! Rewrites names to canonical forms and flattens namespace wrapper statements.
//!
//! Called from:
//! - `crate::pipeline::compile()` after include resolution and before optimization/type checking.
//!
//! Key details:
//! - Builtin fallback and case-insensitive symbol lookup must match PHP visibility rules.

mod expressions;
mod names;
mod declarations;
mod statements;
mod symbols;

use std::collections::{HashMap, HashSet};

use crate::errors::CompileError;
use crate::names::{Name, NameKind};
use crate::parser::ast::{Expr, ExprKind, Program};

/// Tracks namespace use imports for classes, functions, and constants.
/// Used during name resolution to map short names to their canonical fully-qualified names.
#[derive(Default, Clone)]
struct Imports {
    classes: HashMap<String, String>,
    functions: HashMap<String, String>,
    constants: HashMap<String, String>,
}

/// Internal symbol table for tracking declared functions, classes, interfaces, traits,
/// constants, and extern symbols within a namespace scope.
#[derive(Default)]
struct Symbols {
    functions: HashMap<String, String>,
    classes: HashMap<String, String>,
    interfaces: HashMap<String, String>,
    traits: HashMap<String, String>,
    constants: HashSet<String>,
    extern_functions: HashMap<String, String>,
    extern_classes: HashMap<String, String>,
}

/// Resolves PHP namespace/use statements and rewrites names to canonical forms across the program.
pub fn resolve(program: Program) -> Result<Program, CompileError> {
    let mut symbols = Symbols::default();
    symbols::collect_symbols(&program, None, &mut symbols);
    statements::resolve_stmt_list(&program, None, &Imports::default(), &symbols)
}

/// Rewrites string literal arguments for functions that invoke callable names.
/// For functions like `array_map` or `usort`, resolves string callback names to their canonical
/// fully-qualified form using the current namespace and imports. `function_exists()` is excluded
/// because PHP treats its argument as a literal introspection name rather than a callable lookup.
fn rewrite_callback_literal_args(
    function_name: &str,
    args: &[Expr],
    current_namespace: Option<&str>,
    imports: &Imports,
    symbols: &Symbols,
) -> Vec<Expr> {
    let callback_positions: &[usize] = match function_name {
        "call_user_func" | "call_user_func_array" => &[0],
        "array_map" | "array_filter" | "array_reduce" | "array_walk" => &[0],
        "usort" | "uksort" | "uasort" => &[1],
        _ => &[],
    };

    args.iter()
        .enumerate()
        .map(|(idx, arg)| {
            if callback_positions.contains(&idx) {
                if let ExprKind::StringLiteral(raw_name) = &arg.kind {
                    let resolved = names::resolve_function_name(
                        &parse_callback_name(raw_name),
                        current_namespace,
                        imports,
                        symbols,
                    );
                    return Expr::new(ExprKind::StringLiteral(resolved), arg.span);
                }
            }
            arg.clone()
        })
        .collect()
}

/// Parses a string callback name (e.g., `"my_func"` or `"MyNamespace\MyClass::method"`)
/// into a `Name` with the appropriate `NameKind`. Leading backslashes are stripped;
/// names containing backslashes are treated as fully-qualified.
fn parse_callback_name(raw_name: &str) -> Name {
    if let Some(stripped) = raw_name.strip_prefix('\\') {
        return Name::from_parts(
            NameKind::FullyQualified,
            stripped.split('\\').map(str::to_string).collect(),
        );
    }
    if raw_name.contains('\\') {
        return Name::from_parts(
            NameKind::FullyQualified,
            raw_name.split('\\').map(str::to_string).collect(),
        );
    }
    Name::unqualified(raw_name)
}

/// Converts a string containing a fully-qualified name (e.g., `"Namespace\Class"`)
/// into a `Name` with `NameKind::FullyQualified`.
fn resolved_name(name: String) -> Name {
    Name::from_parts(
        NameKind::FullyQualified,
        name.split('\\').map(str::to_string).collect(),
    )
}

/// Extracts the namespace name as a dot-separated string from an optional `Name`.
/// Returns an empty string if the name is `None`.
fn namespace_name(name: &Option<Name>) -> String {
    name.as_ref().map(Name::as_canonical).unwrap_or_default()
}

/// Returns `true` if `name` is a supported builtin function in PHP.
/// Used by name resolution to apply PHP's builtin fallback rules.
pub(crate) fn is_builtin_function(name: &str) -> bool {
    crate::types::checker::builtins::is_supported_builtin_function(name)
}

/// Returns the canonical name for a builtin function, case-normalized.
/// Returns `None` if the name is not a known builtin.
pub(crate) fn canonical_builtin_function_name(name: &str) -> Option<String> {
    crate::types::checker::builtins::canonical_builtin_function_name(name)
}

/// Function names elephc provides as prelude-injected global user functions (see
/// `crate::var_export_prelude`, `crate::shutdown_prelude`, and siblings in `crate::pipeline`),
/// rather than as catalog builtins. PHP considers these always-available global functions, so a
/// bare namespaced call `var_export(...)` / `register_shutdown_function(...)` inside `namespace N`
/// must fall back to the global function — but they are NOT in the builtin catalog (registering
/// them there caused redeclaration/link errors), so the builtin-fallback path in
/// `canonical_function` does not see them.
///
/// Each autoloaded file is name-resolved in isolation by `autoload::load_autoloaded_file`, so the
/// prelude declaration (injected into the main program before the main name-resolution pass) is
/// absent from the per-file symbol table. Seeding these names here keeps the PHP namespace
/// fallback for bare calls correct without re-injecting the prelude per file or duplicating the
/// catalog. Extend this set only with prelude-injected globals that PHP treats as unconditionally
/// available.
///
/// `register_shutdown_function` was added after `crate::shutdown_prelude` shipped (its injection
/// runs, like `var_export_prelude`'s, AFTER the main name-resolution pass — see
/// `crate::pipeline::compile()` — so without an entry here a namespaced unqualified call could
/// not fall back to the prelude's global declaration and hit "Undefined function" instead).
///
/// `is_uploaded_file`/`move_uploaded_file` (`crate::upload_prelude`) and `request_parse_body`
/// (`crate::web_prelude`) are listed for the same reason: Symfony calls them unqualified from
/// inside `namespace Symfony\Component\HttpFoundation[\File]`, and PHP's namespace fallback must
/// find the prelude's global declaration.
const PRELUDE_GLOBAL_FUNCTIONS: &[&str] = &[
    "var_export",
    "register_shutdown_function",
    "parse_ini_file",
    "is_uploaded_file",
    "move_uploaded_file",
    "request_parse_body",
];

/// Returns the canonical name for a prelude-injected global function, case-normalized with a
/// leading `\` stripped. Returns `None` if the name is not a known prelude global. Mirrors
/// `canonical_builtin_function_name` so `Symbols::canonical_function` can fall back to prelude
/// globals exactly as it falls back to catalog builtins.
pub(crate) fn canonical_prelude_global_function_name(name: &str) -> Option<String> {
    let bare = name.trim_start_matches('\\');
    PRELUDE_GLOBAL_FUNCTIONS
        .iter()
        .find(|prelude| bare.eq_ignore_ascii_case(prelude))
        .map(|prelude| (*prelude).to_string())
}

thread_local! {
    /// Global (non-namespaced) function names discovered by `autoload::scan_composer_global_functions`
    /// pre-scanning Composer `autoload.files` entries, keyed by `php_symbol_key`. Installed by
    /// `crate::pipeline::compile()` via `with_known_composer_global_functions` for the span covering
    /// the main name-resolution pass and `autoload::run` (every per-file isolated resolve happens
    /// inside that span too). Empty outside that window, so an unrelated `resolve()` call (e.g. in a
    /// test that does not go through the pipeline) sees no extra fallback names.
    static KNOWN_COMPOSER_GLOBAL_FUNCTIONS: std::cell::RefCell<HashMap<String, String>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Installs `names` (see `autoload::scan_composer_global_functions`) as the known-composer-global-
/// function fallback set for the duration of `f`, then restores whatever was installed before.
///
/// Each Composer-autoloaded file is name-resolved in isolation (`autoload::load_autoloaded_file`),
/// so a namespaced caller in one file cannot see a `function_exists`-guarded global declared in a
/// DIFFERENT `autoload.files` entry through its own (per-file) symbol table alone — the guard and
/// the declaration are visible only within that declaring file's own isolated pass. This mirrors
/// `PRELUDE_GLOBAL_FUNCTIONS` (elephc's own always-available prelude globals) but for names the
/// PROGRAM's own Composer polyfills provide, computed per-compile instead of hardcoded.
pub fn with_known_composer_global_functions<T>(
    names: HashMap<String, String>,
    f: impl FnOnce() -> T,
) -> T {
    KNOWN_COMPOSER_GLOBAL_FUNCTIONS.with(|slot| {
        let previous = slot.replace(names);
        let result = f();
        slot.replace(previous);
        result
    })
}

/// Returns the canonical (originally declared case) name for a known Composer `autoload.files`
/// global function, or `None` when `name` (case/backslash-normalized) is not currently installed.
/// Mirrors `canonical_prelude_global_function_name` so `Symbols::canonical_function` can chain it
/// as one more fallback tier exactly like the prelude-global and builtin tiers.
pub(crate) fn canonical_known_composer_global_function_name(name: &str) -> Option<String> {
    let key = crate::names::php_symbol_key(name.trim_start_matches('\\'));
    KNOWN_COMPOSER_GLOBAL_FUNCTIONS.with(|slot| slot.borrow().get(&key).cloned())
}

/// Reports whether `name` matches one of PHP's procedural date/time aliases
/// (e.g. `date_create`, `idate`, `gmstrftime`). The name set is the same as the one
/// rewritten by `expressions::rewrite_date_procedural_alias`, minus the per-arity guards,
/// so `function_exists()` and other introspection builtins see the same surface that the
/// resolver rewrites.
pub(crate) fn is_date_procedural_alias(name: &str) -> bool {
    expressions::is_date_procedural_alias(name)
}

/// Returns the inclusive `(min, max)` argument arity that the resolver's date/time alias
/// desugaring accepts for `name`, or `None` when `name` is not a desugared alias. The type
/// checker uses this to report a precise arity error (instead of "Undefined function") when a
/// known alias call survives desugaring because its argument count was out of range.
pub(crate) fn date_procedural_alias_arity(name: &str) -> Option<(usize, usize)> {
    expressions::date_procedural_alias_arity(name)
}
