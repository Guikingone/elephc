//! Purpose:
//! Folds closed-world `function_exists('name')` and `function_exists(Name::class)` calls on a
//! string literal or `::class` constant to a boolean literal, so
//! `if (!function_exists('X')) { ... }` guards become constant control flow the existing DCE can
//! prune. This complements the resolver's conditional-function hoisting: once a guarded polyfill
//! wrapper has been hoisted to a real top-level function, its now-redundant in-place guard folds to
//! dead; a guard for a name elephc already provides as a builtin likewise folds away.
//!
//! Called from:
//! - `crate::pipeline::compile()` via `crate::optimize::fold_function_existence` (after type
//!   checking, alongside `fold_class_existence`, before `prune_constant_control_flow`).
//! - `crate::optimize::fold::expr::fold_expr` calls `try_fold_function_exists` on each
//!   `FunctionCall`.
//!
//! Key details:
//! - The fold is deliberately conservative about *runtime load order*. elephc models an
//!   include-discovered user function as loaded at runtime (an include inside a function body), and
//!   `function_exists()` on it must stay a runtime check, not a compile-time constant. Codegen's
//!   `lower_function_exists` (`src/codegen_ir/lower_inst/builtins.rs`) already distinguishes those
//!   (function-variant groups get a runtime check; everything else a static bool), so this fold
//!   folds ONLY the unconditionally-available cases and defers every checked user function to
//!   codegen: a builtin (catalog), an extern, or a date/time procedural alias folds to `true`; a
//!   name that is none of those and not a checked user function folds to `false`; a checked user
//!   function is left unfolded. Matching is case-insensitive with a leading `\` stripped.
//! - Like `class_existence`, folding only happens while the pipeline installs the set; the
//!   pre-type-check `fold_constants` pass sees an empty thread-local slot and leaves calls untouched.

use std::cell::RefCell;
use std::collections::HashSet;

use crate::names::{php_symbol_key, Name};
use crate::parser::ast::{Expr, ExprKind, Program};
use crate::types::CheckResult;

thread_local! {
    /// Closed-world function set installed for the duration of `fold_function_existence`.
    /// `None` outside that pass, so unrelated folds never touch `function_exists` calls.
    static ACTIVE_FUNCTION_EXISTENCE: RefCell<Option<FunctionExistenceSet>> =
        const { RefCell::new(None) };
}

/// Case-insensitive, backslash-stripped view of the checked closed world used to classify a
/// `function_exists('X')` call. Externs are unconditionally available so they fold to `true`;
/// ordinary user functions are recorded separately only to mark a name as "known" (so it is left
/// unfolded rather than folded to `false`), because it may carry runtime load-order semantics.
#[derive(Clone)]
pub struct FunctionExistenceSet {
    user_functions: HashSet<String>,
    externs: HashSet<String>,
}

impl FunctionExistenceSet {
    /// Builds the classification sets from a completed `CheckResult`: checked user functions and
    /// extern functions, each normalized through `normalize_symbol`.
    pub fn from_check_result(check: &CheckResult) -> Self {
        Self {
            user_functions: check.functions.keys().map(|name| normalize_symbol(name)).collect(),
            externs: check.extern_functions.keys().map(|name| normalize_symbol(name)).collect(),
        }
    }

    /// Classifies `name` for folding: `Some(true)` when it is unconditionally available (extern,
    /// PHP-visible builtin, or date/time procedural alias), `Some(false)` when it is genuinely
    /// absent (none of those and not a checked user function), or `None` when it is a checked user
    /// function whose runtime availability must be resolved by codegen rather than folded here.
    fn classify(&self, name: &str) -> Option<bool> {
        let key = normalize_symbol(name);
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

/// Returns whether a callee name is `function_exists`, case-insensitively and with a leading
/// namespace separator stripped (PHP resolves the call to the global builtin).
fn is_function_exists_callee(name: &str) -> bool {
    php_symbol_key(name.trim_start_matches('\\')) == "function_exists"
}

/// Normalizes a symbol name to its closed-world lookup key: lowercase, leading backslashes removed.
fn normalize_symbol(name: &str) -> String {
    php_symbol_key(name.trim_start_matches('\\'))
}
