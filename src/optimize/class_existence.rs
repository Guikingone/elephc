//! Purpose:
//! Folds closed-world `class_exists`/`interface_exists`/`trait_exists`/`enum_exists` calls on a
//! literal (or `::class`) name to a boolean literal, so `class_exists`-guarded blocks referencing
//! absent optional dependencies become constant control flow that the existing DCE can prune.
//!
//! Called from:
//! - `crate::pipeline::compile()` via `crate::optimize::fold_class_existence` (after type checking,
//!   before `prune_constant_control_flow`).
//! - `crate::optimize::fold::expr::fold_expr` calls `try_fold_class_existence` on each `FunctionCall`.
//!
//! Key details:
//! - The fold reuses the constant-folding driver (`fold_block`) so the same pass that turns
//!   `class_exists(Absent::class)` into `false` also collapses the enclosing `!false` to `true`.
//! - The closed-world sets come from `CheckResult` (classes/interfaces/enums) plus AST-collected
//!   trait names, matched case-insensitively with leading backslashes stripped — mirroring the
//!   codegen fold in `src/codegen_ir/lower_inst/builtins.rs`. Synthetic `__elephc*` helper classes
//!   are excluded, exactly as codegen hides them from PHP `class_exists`.
//! - Folding only happens while the pipeline installs the sets; the pre-type-check `fold_constants`
//!   pass sees an empty thread-local slot and leaves these calls untouched.

use std::cell::RefCell;
use std::collections::HashSet;

use crate::names::{php_symbol_key, Name};
use crate::parser::ast::{Expr, ExprKind, Program, StaticReceiver, Stmt, StmtKind};
use crate::types::CheckResult;

thread_local! {
    /// Closed-world existence sets installed for the duration of `fold_class_existence`.
    /// `None` outside that pass, so unrelated folds never touch existence-check calls.
    static ACTIVE_CLASS_EXISTENCE: RefCell<Option<ClassExistenceSets>> = const { RefCell::new(None) };
}

/// Which `*_exists` builtin a `FunctionCall` name maps to.
enum ExistenceKind {
    Class,
    Interface,
    Trait,
    Enum,
}

/// Case-insensitive, backslash-stripped name sets for the closed-world existence check.
/// Every entry is already normalized through `normalize_symbol`.
#[derive(Clone)]
pub struct ClassExistenceSets {
    classes: HashSet<String>,
    interfaces: HashSet<String>,
    traits: HashSet<String>,
    enums: HashSet<String>,
    /// True when the program contains an `eval()` call: eval'd code can declare a
    /// class-like at runtime, so an absent name must NOT fold to `false` (a present
    /// AOT name still folds to `true` — eval can never remove a declaration).
    has_eval: bool,
}

impl ClassExistenceSets {
    /// Builds the closed-world sets from a completed `CheckResult` (classes/interfaces/enums) and
    /// the trait names declared in the program AST. Synthetic `__elephc*` helper classes are
    /// excluded so a user `class_exists("__elephc_...")` cannot fold true, matching codegen.
    pub fn from_program_and_check_result(program: &Program, check: &CheckResult) -> Self {
        let classes = check
            .classes
            .keys()
            .filter(|name| !is_internal_synthetic_class_name(name))
            .map(|name| normalize_symbol(name))
            .collect();
        let interfaces = check
            .interfaces
            .keys()
            .map(|name| normalize_symbol(name))
            .collect();
        let enums = check
            .enums
            .keys()
            .map(|name| normalize_symbol(name))
            .collect();
        let mut traits = HashSet::new();
        collect_declared_trait_names(program, &mut traits);
        let has_eval = program
            .iter()
            .any(crate::ir_lower::stmt_contains_eval_call);
        Self {
            classes,
            interfaces,
            traits,
            enums,
            has_eval,
        }
    }
}

/// Folds closed-world existence checks across `program` and returns the rewritten AST.
///
/// Installs `sets` in the thread-local slot for the duration of the pass, then runs the shared
/// constant-folding driver so every `class_exists`/`interface_exists`/`trait_exists`/`enum_exists`
/// call on a literal/`::class` name collapses to a boolean (and the enclosing `!`/`&&`/`||` folds
/// with it), before restoring the previous slot. Covers top-level statements and function bodies,
/// which EIR lowering reads from the program AST.
pub fn fold_class_existence(program: Program, sets: &ClassExistenceSets) -> Program {
    ACTIVE_CLASS_EXISTENCE.with(|slot| {
        let previous = slot.replace(Some(sets.clone()));
        let result = super::control::fold_block(program);
        slot.replace(previous);
        result
    })
}

/// Folds closed-world existence checks in every class/enum method body stored in `check`.
///
/// EIR lowering re-sources class and enum method bodies from `check.classes[..].method_decls`
/// (the checker's trait-flattened, `self`-resolved declarations captured at type-check time),
/// not from the optimized program AST, so a `class_exists`-guarded block inside a method would
/// otherwise reach codegen unfolded. This installs `sets` once and folds each method body in
/// place, so a folded constant-true guard lets EIR lowering's own constant-branch/divergent-tail
/// pruning drop the guarded `new AbsentClass`. Enums are covered because they carry a `ClassInfo`
/// in `check.classes`.
pub fn fold_class_existence_in_method_bodies(check: &mut CheckResult, sets: &ClassExistenceSets) {
    ACTIVE_CLASS_EXISTENCE.with(|slot| {
        let previous = slot.replace(Some(sets.clone()));
        for class_info in check.classes.values_mut() {
            for method in class_info.method_decls.iter_mut() {
                let body = std::mem::take(&mut method.body);
                method.body = super::control::fold_block(body);
            }
        }
        slot.replace(previous);
    });
}

/// Attempts to fold a `FunctionCall` to a boolean when it is a closed-world existence check on a
/// literal/`::class` class name. Returns `None` (leaving the call intact) when the existence sets
/// are not installed, the callee is not an existence builtin, the name argument is not a static
/// literal, or a trailing argument (the `autoload` flag) could carry an observable side effect.
pub(in crate::optimize) fn try_fold_class_existence(name: &Name, args: &[Expr]) -> Option<ExprKind> {
    ACTIVE_CLASS_EXISTENCE.with(|slot| {
        let borrowed = slot.borrow();
        let sets = borrowed.as_ref()?;
        let kind = classify_existence_builtin(name.as_str())?;
        let literal = args.first().and_then(static_name_from_literal_or_class_const)?;
        // Folding the call away would drop the evaluation of any further arguments, so only fold
        // when the trailing `autoload` argument (if present) is a side-effect-free literal.
        if args[1..].iter().any(|arg| !is_side_effect_free_literal(arg)) {
            return None;
        }
        let key = normalize_symbol(&literal);
        let set = match kind {
            ExistenceKind::Class => &sets.classes,
            ExistenceKind::Interface => &sets.interfaces,
            ExistenceKind::Trait => &sets.traits,
            ExistenceKind::Enum => &sets.enums,
        };
        let exists = set.contains(&key);
        // With an `eval()` in the program, a currently-absent class-like may be declared
        // at runtime by eval'd code (the EIR lowering probes the eval class table for it),
        // so only the sound direction folds: present names fold `true`, absent names stay
        // a live call.
        if !exists && sets.has_eval {
            return None;
        }
        Some(ExprKind::BoolLiteral(exists))
    })
}

/// Maps a callee name to its existence-check kind, case-insensitively with a leading `\` stripped.
fn classify_existence_builtin(name: &str) -> Option<ExistenceKind> {
    match php_symbol_key(name.trim_start_matches('\\')).as_str() {
        "class_exists" => Some(ExistenceKind::Class),
        "interface_exists" => Some(ExistenceKind::Interface),
        "trait_exists" => Some(ExistenceKind::Trait),
        "enum_exists" => Some(ExistenceKind::Enum),
        _ => None,
    }
}

/// Extracts a static name from an existence-check first argument: a string literal or a
/// `Name::class` constant. Returns `None` for dynamic names or `self`/`static`/`parent::class`,
/// which are left to runtime/codegen lowering.
///
/// The returned string is the canonical name as it appears after `name_resolver` canonicalization,
/// so for `Name::class` inside a namespace block it is the fully-qualified name. Shared with
/// `function_existence` so `function_exists(Name::class)` folds against the same `::class`-to-FQN
/// resolution that `class_exists(Name::class)` uses, without forking the resolver.
pub(in crate::optimize) fn static_name_from_literal_or_class_const(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::StringLiteral(value) => Some(value.clone()),
        ExprKind::ClassConstant {
            receiver: StaticReceiver::Named(name),
        } => Some(name.as_canonical()),
        _ => None,
    }
}

/// Returns true when `expr` is a scalar/`::class` literal that cannot carry a side effect, so
/// folding away a call that evaluates it does not drop observable behavior.
fn is_side_effect_free_literal(expr: &Expr) -> bool {
    matches!(
        &expr.kind,
        ExprKind::StringLiteral(_)
            | ExprKind::IntLiteral(_)
            | ExprKind::FloatLiteral(_)
            | ExprKind::BoolLiteral(_)
            | ExprKind::Null
            | ExprKind::ClassConstant { .. }
    )
}

/// Normalizes a symbol name to its closed-world lookup key: lowercase, leading backslashes removed.
fn normalize_symbol(name: &str) -> String {
    php_symbol_key(name.trim_start_matches('\\'))
}

/// Returns true for internal helper classes hidden from PHP `class_exists()` (mirrors codegen).
fn is_internal_synthetic_class_name(name: &str) -> bool {
    php_symbol_key(name).starts_with("__elephc")
}

/// Collects user-declared trait names (including inside namespace blocks) into `out`, normalized.
fn collect_declared_trait_names(stmts: &[Stmt], out: &mut HashSet<String>) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::TraitDecl { name, .. } => {
                out.insert(normalize_symbol(name));
            }
            StmtKind::NamespaceBlock { body, .. } => collect_declared_trait_names(body, out),
            _ => {}
        }
    }
}
