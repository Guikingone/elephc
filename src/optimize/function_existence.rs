//! Purpose:
//! Folds closed-world `function_exists('name')` and `function_exists(Name::class)` calls on a
//! string literal or `::class` constant to a boolean literal, so
//! `if (!function_exists('X')) { ... }` guards become constant control flow the existing DCE can
//! prune. This complements the resolver's conditional-function hoisting: once a guarded polyfill
//! wrapper has been hoisted to a real top-level function, its now-redundant in-place guard folds to
//! dead; a guard for a name elephc already provides as a builtin likewise folds away.
//!
//! A second, narrower mode runs BEFORE type checking (`FunctionExistenceSet::for_pre_check`): it
//! FALSE-folds `function_exists`/`extension_loaded` on a literal name ONLY for a curated allowlist
//! of PHP extensions elephc never provides (`fastcgi_*`, `litespeed_*`, `igbinary_*`,
//! `frankenphp_*`, `apcu_*`, `opcache_*`, `xdebug_*`), and only when the program does not itself
//! declare a same-named function (a real user/polyfill definition always wins). This lets a
//! Composer runtime's `if (function_exists('fastcgi_finish_request')) { fastcgi_finish_request(); }`
//! guard collapse to dead code before the checker ever sees the call to an extension function
//! elephc cannot resolve, instead of failing with "Undefined function".
//!
//! The same pre-check window also folds the *version* form of the very same question:
//! `phpversion('<curated extension>')` to `false`, and `version_compare(false|'', '<version>',
//! '<operator>')` to the boolean PHP itself computes. Together they collapse
//! `if (version_compare(phpversion('redis'), '6.2.0', '>=')) { trait T {...} } else { trait T {...} }`
//! -- Symfony Cache's conditional Redis/Relay proxy traits -- to the else-branch declaration, which
//! is exactly what PHP declares on a runtime without those extensions.
//!
//! Called from:
//! - `crate::pipeline::compile()` via `crate::optimize::fold_function_existence` (twice: once in
//!   pre-check mode before type checking, and once in the original post-check mode afterward,
//!   alongside `fold_class_existence`, before `prune_constant_control_flow`).
//! - `crate::optimize::fold::expr::fold_expr` calls `try_fold_function_exists`,
//!   `try_fold_extension_loaded_pre_check`, `try_fold_phpversion_pre_check`, and
//!   `try_fold_version_compare_pre_check` on each `FunctionCall`, and the pre-check-only
//!   short-circuit helpers on `&&`/`||`/ternary/short-ternary nodes.
//!
//! Key details:
//! - The post-check fold is deliberately conservative about *runtime load order*. elephc models an
//!   include-discovered user function as loaded at runtime (an include inside a function body), and
//!   `function_exists()` on it must stay a runtime check, not a compile-time constant. Codegen's
//!   `lower_function_exists` (`src/codegen_ir/lower_inst/builtins.rs`) already distinguishes those
//!   (function-variant groups get a runtime check; everything else a static bool), so this fold
//!   folds ONLY the unconditionally-available cases and defers every checked user function to
//!   codegen: a builtin (catalog), an extern, or a date/time procedural alias folds to `true`; a
//!   name that is none of those and not a checked user function folds to `false`; a checked user
//!   function is left unfolded. Matching is case-insensitive with a leading `\` stripped.
//! - The pre-check fold NEVER folds to `true` (JURY ADDENDUM #1): it only ever proves a curated
//!   extension name absent. Everything else — including real builtins, which the post-check pass
//!   already true-folds once `CheckResult` exists — is left untouched pre-checker. The
//!   `version_compare` fold is the one place a `true` can be produced, and only as the arithmetic
//!   consequence of an already-proven-absent extension ("" is less than every real version, so
//!   `version_compare('', '6.2.0', '<')` is `true` in PHP too) — never as a claim of availability.
//! - Like `class_existence`, folding only happens while the pipeline installs the set; a
//!   `fold_constants`/`fold_function_existence` call outside either window sees an empty
//!   thread-local slot and leaves calls untouched.
//! - The pre-check short-circuit helpers (`try_fold_precheck_short_circuit`,
//!   `try_fold_precheck_ternary`, `try_fold_precheck_short_ternary`) are separate from the general
//!   `fold/ops.rs` literal folders on purpose: PHP guarantees `&&`/`||` short-circuit and a ternary
//!   never evaluates its untaken branch, so once our OWN curated fold has produced a `BoolLiteral`
//!   condition, the other operand/branch can be dropped unconditionally — even when it is not
//!   itself a pure/literal expression (e.g. `fastcgi_finish_request()`). The general fold engine
//!   deliberately does not take this shortcut (it requires both operands/branches to already be
//!   literal, or the dropped side to be side-effect-free) because it runs unconditionally on every
//!   program; gating the aggressive drop behind the pre-check thread-local keeps that general
//!   behavior completely unchanged and confines the new aggressiveness to our curated, closed-world
//!   window.

use std::cell::RefCell;
use std::collections::HashSet;

use crate::names::{php_symbol_key, Name};
use crate::parser::ast::{BinOp, Expr, ExprKind, Program, Stmt, StmtKind};
use crate::types::CheckResult;

thread_local! {
    /// Closed-world function set installed for the duration of `fold_function_existence`.
    /// `None` outside that pass, so unrelated folds never touch `function_exists` calls.
    static ACTIVE_FUNCTION_EXISTENCE: RefCell<Option<FunctionExistenceSet>> =
        const { RefCell::new(None) };
}

/// Distinguishes the two fold windows sharing `ACTIVE_FUNCTION_EXISTENCE`.
///
/// `PostCheck` is the original, unmodified behavior: it runs after type checking with a completed
/// `CheckResult` and can both true- and false-fold. `PreCheck` runs before type checking with only
/// a lightweight program scan available; it can only prove a curated extension name absent
/// (JURY ADDENDUM #1: never true-fold pre-checker) and additionally licenses the aggressive
/// short-circuit helpers below.
#[derive(Clone, Copy, PartialEq, Eq)]
enum FoldMode {
    PreCheck,
    PostCheck,
}

/// PHP extension function-name prefixes elephc never provides and never will emulate as a real
/// builtin, matched case-insensitively. Deliberately narrow: `pcntl_` is excluded even though the
/// pcntl extension is largely unimplemented, because elephc DOES implement a handful of real
/// `pcntl_*` builtins (`pcntl_alarm`, `pcntl_async_signals`, `pcntl_signal`,
/// `pcntl_signal_get_handler` — see `src/types/checker/builtins/catalog.rs`); a blanket `pcntl_`
/// prefix would false-fold `function_exists('pcntl_alarm')`, which is silently wrong. Extend this
/// list only for prefixes elephc has zero catalog presence under, so a prefix match can never
/// collide with a real (present or future) builtin.
const NEVER_AVAILABLE_FUNCTION_PREFIXES: &[&str] = &[
    "apcu_",
    "opcache_",
    "xdebug_",
    "igbinary_",
    "fastcgi_",
    "litespeed_",
    "frankenphp_",
];

/// PHP extension names elephc never reports as loaded, matched case-insensitively and exactly
/// (extension names, unlike function names, are single words rather than prefixed families).
/// Kept in lockstep with `NEVER_AVAILABLE_FUNCTION_PREFIXES`'s families that are genuinely queried
/// through `extension_loaded()` in the wild (e.g. `symfony/cache`'s `DefaultMarshaller` checks
/// `extension_loaded('igbinary')` before calling `igbinary_serialize()`), plus the extensions whose
/// *version* is queried through `phpversion('<ext>')` (`symfony/cache`'s Redis/Relay proxies gate
/// their trait declarations on `version_compare(phpversion('redis'), …)`). elephc ships no loadable
/// extension of any kind, so both answers are already what its own runtime produces: codegen's
/// `lower_extension_loaded` returns `false` unconditionally and `lower_phpversion` returns `false`
/// for every one-argument form. Entries are added only for extensions elephc has no catalog
/// presence under, so folding can never contradict a real (present or future) builtin family.
const NEVER_AVAILABLE_EXTENSIONS: &[&str] =
    &["apcu", "opcache", "xdebug", "igbinary", "redis", "relay"];

/// Case-insensitive, backslash-stripped view of the checked closed world used to classify a
/// `function_exists('X')` call. Externs are unconditionally available so they fold to `true`;
/// ordinary user functions are recorded separately only to mark a name as "known" (so it is left
/// unfolded rather than folded to `false`), because it may carry runtime load-order semantics.
#[derive(Clone)]
pub struct FunctionExistenceSet {
    user_functions: HashSet<String>,
    externs: HashSet<String>,
    mode: FoldMode,
}

impl FunctionExistenceSet {
    /// Builds the classification sets from a completed `CheckResult`: checked user functions and
    /// extern functions, each normalized through `normalize_symbol`. Runs in `PostCheck` mode.
    pub fn from_check_result(check: &CheckResult) -> Self {
        Self {
            user_functions: check.functions.keys().map(|name| normalize_symbol(name)).collect(),
            externs: check.extern_functions.keys().map(|name| normalize_symbol(name)).collect(),
            mode: FoldMode::PostCheck,
        }
    }

    /// Builds a pre-checker classification set from a raw, name-resolved (but not yet type-checked)
    /// `Program`. There is no `CheckResult` yet, so `externs` stays empty (irrelevant to the curated
    /// extension-function allowlist this mode consults) and `user_functions` is populated by a
    /// lightweight recursive scan for every declared free-function name in the program (top-level
    /// and namespace-block-nested `FunctionDecl`/`FunctionVariantGroup`/`ExternFunctionDecl`
    /// declarations), so a program that itself defines e.g. `function igbinary_serialize(...)` (a
    /// real polyfill, not merely a guard) is never false-folded. Runs in `PreCheck` mode, which
    /// `classify` restricts to false-only curated-allowlist decisions.
    pub fn for_pre_check(program: &Program) -> Self {
        let mut user_functions = HashSet::new();
        collect_declared_function_names(program, &mut user_functions);
        Self {
            user_functions,
            externs: HashSet::new(),
            mode: FoldMode::PreCheck,
        }
    }

    /// Classifies `name` for folding: `Some(true)` when it is unconditionally available (extern,
    /// PHP-visible builtin, or date/time procedural alias), `Some(false)` when it is genuinely
    /// absent (none of those and not a checked user function), or `None` when it is a checked user
    /// function whose runtime availability must be resolved by codegen rather than folded here.
    ///
    /// In `PreCheck` mode this never returns `Some(true)`: it only proves a curated
    /// never-available extension function absent, and only when the program does not itself
    /// declare a same-named function.
    fn classify(&self, name: &str) -> Option<bool> {
        let key = normalize_symbol(name);
        match self.mode {
            FoldMode::PreCheck => {
                if is_never_available_function(&key) && !self.user_functions.contains(&key) {
                    Some(false)
                } else {
                    None
                }
            }
            FoldMode::PostCheck => {
                if self.externs.contains(&key)
                    || crate::types::checker::builtins::is_php_visible_builtin_function(&key)
                    || crate::name_resolver::is_date_procedural_alias(&key)
                {
                    Some(true)
                } else if self.user_functions.contains(&key) {
                    None
                } else {
                    Some(false)
                }
            }
        }
    }

    /// Classifies an `extension_loaded('X')` argument in `PreCheck` mode only: `Some(false)` when
    /// `name` is a curated never-loaded extension not shadowed by a same-named user function
    /// (extensions and functions do not share a namespace in PHP, but a program defining e.g.
    /// `function extension_loaded_shim_igbinary()` is irrelevant here — the shadow check mirrors
    /// `classify`'s caution even though no real collision is possible), `None` otherwise. Codegen's
    /// `lower_extension_loaded` (`src/codegen_ir/lower_inst/builtins.rs`) already folds every
    /// `extension_loaded()` call to `false` unconditionally at codegen time (elephc has no
    /// dynamically loaded extensions), so this pre-check classifier only needs to cover the
    /// narrower job of getting dead branches out of the checker's way; it never needs a `PostCheck`
    /// counterpart.
    fn classify_extension(&self, name: &str) -> Option<bool> {
        if self.mode != FoldMode::PreCheck {
            return None;
        }
        let key = name.trim_start_matches('\\').to_ascii_lowercase();
        if NEVER_AVAILABLE_EXTENSIONS.iter().any(|candidate| *candidate == key) {
            Some(false)
        } else {
            None
        }
    }
}

/// Returns whether `key` (already normalized: lowercase, no leading `\`) matches a curated
/// never-available extension function prefix.
fn is_never_available_function(key: &str) -> bool {
    NEVER_AVAILABLE_FUNCTION_PREFIXES
        .iter()
        .any(|prefix| key.starts_with(prefix))
}

/// Recursively collects every declared free-function name (lowercase, `php_symbol_key`-normalized)
/// reachable in `stmts` into `out`. Covers `FunctionDecl`, `FunctionVariantGroup`, and
/// `ExternFunctionDecl` at top level and inside `NamespaceBlock` nesting — the only shapes a free
/// function declaration can take by the time this pre-checker pass runs (conditionally-nested
/// declarations have already been hoisted to one of these top-level/namespace-block shapes by
/// `resolver::hoist_conditional_function_declarations`, which runs earlier in the pipeline).
/// Deliberately does not descend into class/enum/interface/trait method bodies: PHP methods are
/// never candidates for `function_exists()`, and a curated extension name is a free-function name.
fn collect_declared_function_names(stmts: &[Stmt], out: &mut HashSet<String>) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::FunctionDecl { name, .. } => {
                out.insert(normalize_symbol(name));
            }
            StmtKind::FunctionVariantGroup { name, .. } => {
                out.insert(normalize_symbol(name));
            }
            StmtKind::ExternFunctionDecl { name, .. } => {
                out.insert(normalize_symbol(name));
            }
            StmtKind::NamespaceBlock { body, .. } => collect_declared_function_names(body, out),
            _ => {}
        }
    }
}

/// Folds closed-world `function_exists` checks across `program` and returns the rewritten AST.
///
/// Installs `set` in the thread-local slot for the duration of the pass, then runs the shared
/// constant-folding driver so every `function_exists('X')` call on a string literal collapses to a
/// boolean (and the enclosing `!`/`&&`/`||` folds with it), before restoring the previous slot.
pub fn fold_function_existence(program: Program, set: &FunctionExistenceSet) -> Program {
    ACTIVE_FUNCTION_EXISTENCE.with(|slot| {
        let previous = slot.replace(Some(set.clone()));
        let result = super::control::fold_block(program);
        slot.replace(previous);
        result
    })
}

/// Folds closed-world `function_exists` checks in every class/enum method body stored in `check`.
///
/// EIR lowering re-sources method bodies from `check.classes[..].method_decls`, not the optimized
/// program AST, so a `function_exists`-guarded block inside a method would otherwise reach codegen
/// unfolded. This installs `set` once and folds each method body in place, mirroring
/// `fold_class_existence_in_method_bodies`.
pub fn fold_function_existence_in_method_bodies(check: &mut CheckResult, set: &FunctionExistenceSet) {
    ACTIVE_FUNCTION_EXISTENCE.with(|slot| {
        let previous = slot.replace(Some(set.clone()));
        for class_info in check.classes.values_mut() {
            for method in class_info.method_decls.iter_mut() {
                let body = std::mem::take(&mut method.body);
                method.body = super::control::fold_block(body);
            }
        }
        slot.replace(previous);
    });
}

/// Returns whether the pre-check fold window is currently installed. Gates the aggressive
/// short-circuit/ternary helpers below so they can only ever fire during the dedicated pre-checker
/// pass, never during ordinary `fold_constants` or the post-check `function_exists` fold.
fn pre_check_mode_active() -> bool {
    ACTIVE_FUNCTION_EXISTENCE.with(|slot| {
        matches!(
            slot.borrow().as_ref().map(|set| set.mode),
            Some(FoldMode::PreCheck)
        )
    })
}

/// Attempts to fold a `FunctionCall` to a boolean when it is a closed-world `function_exists` check
/// on a single name argument. The argument may be a string literal (`function_exists('X')`) or a
/// `Name::class` constant (`function_exists(X::class)`), the latter resolved to its FQN through the
/// shared `static_name_from_literal_or_class_const` resolver so `function_exists` and `class_exists`
/// use one `::class`-to-FQN path.
///
/// Returns `None` (leaving the call intact) when the set is not installed, the callee is not
/// `function_exists`, or the argument is not a lone literal/`::class`. When the resolved name
/// classifies to a boolean, the call folds to `BoolLiteral`. When the name is a checked user
/// function whose runtime availability must be deferred to codegen (`classify` returns `None`) AND
/// the argument was a `::class` constant, the call is rewritten in place to use the resolved FQN
/// string literal so codegen's `lower_function_exists` receives a `const_string_operand` it can
/// lower (otherwise codegen rejects `function_exists` with a non-literal function name). A string-
/// literal argument that defers is left untouched, since codegen already handles it.
pub(in crate::optimize) fn try_fold_function_exists(name: &Name, args: &[Expr]) -> Option<ExprKind> {
    ACTIVE_FUNCTION_EXISTENCE.with(|slot| {
        let borrowed = slot.borrow();
        let set = borrowed.as_ref()?;
        if !is_function_exists_callee(name.as_str()) {
            return None;
        }
        let [arg] = args else {
            return None;
        };
        let literal = super::class_existence::static_name_from_literal_or_class_const(arg)?;
        match set.classify(&literal) {
            Some(bool) => Some(ExprKind::BoolLiteral(bool)),
            None => {
                // A deferred user function: codegen must lower the call with a runtime/variant
                // check. Only rewrite when the argument was a `::class` constant — a string-literal
                // argument is already lowerable by codegen, so leave it intact.
                if matches!(&arg.kind, ExprKind::ClassConstant { .. }) {
                    Some(ExprKind::FunctionCall {
                        name: name.clone(),
                        args: vec![Expr::new(ExprKind::StringLiteral(literal), arg.span)],
                    })
                } else {
                    None
                }
            }
        }
    })
}

/// Attempts to fold an `extension_loaded('X')` call to `BoolLiteral(false)` when the pre-check
/// window is installed and `X` is a literal string naming a curated never-loaded extension not
/// shadowed by a same-named user function. Returns `None` in every other case — in particular it is
/// a no-op outside the pre-check window, so it never interferes with codegen's own unconditional
/// `extension_loaded` → `false` fold (see `classify_extension`'s docs).
pub(in crate::optimize) fn try_fold_extension_loaded_pre_check(
    name: &Name,
    args: &[Expr],
) -> Option<ExprKind> {
    ACTIVE_FUNCTION_EXISTENCE.with(|slot| {
        let borrowed = slot.borrow();
        let set = borrowed.as_ref()?;
        if php_symbol_key(name.as_str().trim_start_matches('\\')) != "extension_loaded" {
            return None;
        }
        let [arg] = args else {
            return None;
        };
        let ExprKind::StringLiteral(literal) = &arg.kind else {
            return None;
        };
        set.classify_extension(literal).map(ExprKind::BoolLiteral)
    })
}

/// Attempts to fold `phpversion('X')` to `BoolLiteral(false)` when the pre-check window is
/// installed and `X` is a literal string naming a curated never-loaded extension.
///
/// PHP returns the extension's version string when the extension is loaded and `false` when it is
/// not; elephc loads no extensions at all, and codegen's `lower_phpversion`
/// (`src/codegen/lower_inst/builtins.rs`) already emits `false` for every one-argument form. This
/// fold therefore only moves elephc's own, already-final answer earlier — from codegen to before
/// the type checker — so a version guard around a conditional declaration becomes constant control
/// flow `prune_dead_static_branches` can resolve. Returns `None` for the zero-argument form (which
/// yields the runtime version string, not a boolean), for a non-literal argument, and outside the
/// pre-check window.
pub(in crate::optimize) fn try_fold_phpversion_pre_check(
    name: &Name,
    args: &[Expr],
) -> Option<ExprKind> {
    ACTIVE_FUNCTION_EXISTENCE.with(|slot| {
        let borrowed = slot.borrow();
        let set = borrowed.as_ref()?;
        if php_symbol_key(name.as_str().trim_start_matches('\\')) != "phpversion" {
            return None;
        }
        let [arg] = args else {
            return None;
        };
        let ExprKind::StringLiteral(literal) = &arg.kind else {
            return None;
        };
        set.classify_extension(literal).map(ExprKind::BoolLiteral)
    })
}

/// Attempts to fold `version_compare($absent, '<version>', '<operator>')` to the boolean PHP
/// computes, where `$absent` is the `false` (or `''`) an absent extension's `phpversion()` yields.
///
/// PHP coerces the `false` first argument to `''`, and `''` compares as strictly LESS THAN every
/// version string containing an alphanumeric character (verified against `php -n` 8.5.6:
/// `version_compare('', '6.2.0')` is `-1`, as is `''` against `a`, `0`, `0.0`, `dev`). The
/// alphanumeric requirement is deliberate: PHP canonicalizes separators away, so a second argument
/// made only of them (`"\0"`) canonicalizes to `''` and compares EQUAL — folding those would be
/// wrong, so they are left alone. The operator must be one of PHP's literal comparison spellings,
/// matched case-SENSITIVELY as PHP does (`'GE'` is not `'ge'`); anything else is left unfolded so
/// the `ValueError` PHP 8 raises for an invalid operator still happens, as is the two-argument form
/// (which returns `-1|0|1` rather than a boolean).
///
/// Returns `None` outside the pre-check window, so this never changes an ordinary `fold_constants`
/// run.
pub(in crate::optimize) fn try_fold_version_compare_pre_check(
    name: &Name,
    args: &[Expr],
) -> Option<ExprKind> {
    if !pre_check_mode_active() {
        return None;
    }
    if php_symbol_key(name.as_str().trim_start_matches('\\')) != "version_compare" {
        return None;
    }
    let [left, right, operator] = args else {
        return None;
    };
    if !is_absent_extension_version(left) {
        return None;
    }
    let ExprKind::StringLiteral(right) = &right.kind else {
        return None;
    };
    if !right.chars().any(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let ExprKind::StringLiteral(operator) = &operator.kind else {
        return None;
    };
    // The left operand canonicalizes to the empty version, which orders strictly before `right`.
    version_compare_less_than_result(operator).map(ExprKind::BoolLiteral)
}

/// Returns whether an expression is the version value PHP yields for an absent extension: the
/// literal `false` our own `phpversion` fold produces, or the empty string it coerces to.
fn is_absent_extension_version(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::BoolLiteral(false) => true,
        ExprKind::StringLiteral(literal) => literal.is_empty(),
        _ => false,
    }
}

/// Maps a `version_compare()` operator literal onto the boolean PHP returns when the left version
/// sorts strictly BEFORE the right one. Mirrors PHP's accepted spellings exactly (`<`/`lt`,
/// `<=`/`le`, `>`/`gt`, `>=`/`ge`, `==`/`=`/`eq`, `!=`/`<>`/`ne`); every other operator raises a
/// `ValueError` in PHP 8, so it stays unfolded and reaches the runtime unchanged.
fn version_compare_less_than_result(operator: &str) -> Option<bool> {
    match operator {
        "<" | "lt" | "<=" | "le" | "!=" | "<>" | "ne" => Some(true),
        ">" | "gt" | ">=" | "ge" | "==" | "=" | "eq" => Some(false),
        _ => None,
    }
}

/// Returns the operand's boolean value ONLY when it is already a literal `BoolLiteral` node — the
/// narrow shape our own curated folds (`try_fold_function_exists`, `try_fold_extension_loaded_pre_check`)
/// produce. Deliberately does not consult general scalar truthiness (unlike `fold/ops.rs`'s
/// `scalar_value`) so the aggressive drop below only ever fires on conditions we ourselves proved,
/// never on arbitrary int/string/float literal truthiness a user happened to write.
fn precheck_bool_literal(expr: &Expr) -> Option<bool> {
    match &expr.kind {
        ExprKind::BoolLiteral(value) => Some(*value),
        _ => None,
    }
}

/// Pre-check-only short-circuit fold for `&&`/`||`: when `left` is a proven `BoolLiteral` that
/// alone decides the result (`false` for `&&`, `true` for `||`), returns that boolean and drops
/// `right` unconditionally — sound regardless of `right`'s purity because PHP itself guarantees
/// `right` is never evaluated in that case. Returns `None` outside the pre-check window (so the
/// general `fold/ops.rs::try_fold_binary_op` — which requires both operands to already be scalar
/// literals — is the only `&&`/`||` folder anywhere else) or when `left` does not decide the
/// outcome by itself.
pub(in crate::optimize) fn try_fold_precheck_short_circuit(
    op: &BinOp,
    left: &Expr,
    right: &Expr,
) -> Option<ExprKind> {
    if !pre_check_mode_active() {
        return None;
    }
    let _ = right; // dropped by design: PHP never evaluates it once `left` decides the result.
    match (op, precheck_bool_literal(left)) {
        (BinOp::And, Some(false)) => Some(ExprKind::BoolLiteral(false)),
        (BinOp::Or, Some(true)) => Some(ExprKind::BoolLiteral(true)),
        _ => None,
    }
}

/// Pre-check-only ternary fold: when `condition` is a proven `BoolLiteral`, returns the taken
/// branch's kind and drops the untaken branch unconditionally (PHP never evaluates it). Returns
/// `None` outside the pre-check window or when `condition` is not a proven literal.
pub(in crate::optimize) fn try_fold_precheck_ternary(
    condition: &Expr,
    then_expr: &Expr,
    else_expr: &Expr,
) -> Option<ExprKind> {
    if !pre_check_mode_active() {
        return None;
    }
    match precheck_bool_literal(condition) {
        Some(true) => Some(then_expr.kind.clone()),
        Some(false) => Some(else_expr.kind.clone()),
        None => None,
    }
}

/// Pre-check-only short-ternary (`?:`) fold: when `value` is a proven `BoolLiteral`, returns
/// `value` itself when truthy or `default` when falsy, dropping the untaken side unconditionally.
/// Returns `None` outside the pre-check window or when `value` is not a proven literal.
pub(in crate::optimize) fn try_fold_precheck_short_ternary(
    value: &Expr,
    default: &Expr,
) -> Option<ExprKind> {
    if !pre_check_mode_active() {
        return None;
    }
    match precheck_bool_literal(value) {
        Some(true) => Some(value.kind.clone()),
        Some(false) => Some(default.kind.clone()),
        None => None,
    }
}

/// Returns whether a callee name is `function_exists`, case-insensitively and with a leading
/// namespace separator stripped (PHP resolves the call to the global builtin).
fn is_function_exists_callee(name: &str) -> bool {
    php_symbol_key(name.trim_start_matches('\\')) == "function_exists"
}

/// Normalizes a symbol name to its closed-world lookup key: lowercase, leading backslashes removed.
fn normalize_symbol(name: &str) -> String {
    php_symbol_key(name.trim_start_matches('\\'))
}
