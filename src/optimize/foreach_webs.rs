//! Purpose:
//! Gives a `foreach` binding that starts a disjoint live range of an already-used local its own
//! name, so the checker can type each range on its own instead of widening the shared slot.
//!
//! Called from:
//! - `crate::pipeline` after constant folding, before integer-range analysis and type checking,
//!   only when EIR optimizations are enabled.
//!
//! Key details:
//! - `foreach ($words as $w) { ... } ... foreach ($counts as $w => $c) { echo $w; }` binds `$w`
//!   to a string first and to an `int|string` key later. One frame slot for both forces boxed
//!   `Mixed` storage for every read of the first loop. When the later loop is the last mention of
//!   the name, its binding and body can use a fresh name without any observable difference.
//! - Only a `foreach` that is a direct statement of a function body or of the top-level program
//!   qualifies (inside a loop, earlier statements would run again after it), and the name must
//!   not appear in its array expression or anywhere after it.
//! - Each iteration of the renamed loop releases the earlier range's slot (recorded here and
//!   applied by EIR lowering), so its value is dropped at the point the original binding would
//!   have overwritten it, keeping destructor order; an empty iteration leaves it alive as before.
//!   An AST `unset()` would do the same but make the slot nullable, defeating the split.
//! - Only a `foreach` KEY is split (PHP binds the value first, the key second, and the release
//!   follows both), and never a superglobal, an `extern global`, a parameter or `$this`.
//! - The fresh slot is declared right after the original one (`LoweringContext::declare_local`),
//!   so frame cleanup and unwinding destroy the later range's last value at the original
//!   variable's position, as PHP does.
//! - The pass refuses any body with references, `global`/`static`, closures, `eval`, includes,
//!   scope-introspection builtins, `$GLOBALS`, `goto`, or the name passed directly as a call
//!   argument before the loop (a by-reference callee could keep an alias). The checks read the
//!   statements' `Debug` text, so they can only refuse too often, never too rarely.
//! - Diagnostics print the PHP name: unused-variable analysis and message rendering map the
//!   fresh name back through [`source_variable_name`].
//! - After renaming, the loop's text must not mention the old name at all, so a variable
//!   position the walker does not route through the rename makes the pass keep the original.

use crate::magic_constants::walker::{walk_program, Pass};
use crate::parser::ast::{ExprKind, MagicConstant, Program, Stmt, StmtKind};
use crate::span::Span;

/// Splits disjoint trailing `foreach` live ranges in the program and its function bodies.
pub fn split_foreach_webs(program: Program) -> Program {
    let program_text = format!("{program:?}");
    let facts = ProgramFacts {
        top_level_shared: program_text.contains("Global {") || program_text.contains("GLOBALS"),
        extern_globals: collect_extern_globals(&program),
    };
    let program = split_body(program, &[], true, &facts);
    program.into_iter().map(|stmt| split_function_decls(stmt, &facts)).collect()
}

/// Collects `extern global` names declared at the top level or inside braced namespaces.
fn collect_extern_globals(stmts: &[Stmt]) -> Vec<String> {
    stmts.iter().flat_map(|stmt| match &stmt.kind {
        StmtKind::ExternGlobalDecl { name, .. } => vec![name.clone()],
        StmtKind::NamespaceBlock { body, .. } => collect_extern_globals(body),
        _ => Vec::new(),
    }).collect()
}

/// Recurses into top-level function declarations (also inside braced namespaces).
fn split_function_decls(stmt: Stmt, facts: &ProgramFacts) -> Stmt {
    match &stmt.kind {
        StmtKind::FunctionDecl { .. } => {
            let mut stmt = stmt;
            if let StmtKind::FunctionDecl { params, variadic, body, .. } = &mut stmt.kind {
                let mut names: Vec<String> = params.iter().map(|param| param.0.clone()).collect();
                names.extend(variadic.iter().cloned());
                let taken = std::mem::take(body);
                *body = split_body(taken, &names, false, facts);
            }
            stmt
        }
        StmtKind::NamespaceBlock { .. } => {
            let mut stmt = stmt;
            if let StmtKind::NamespaceBlock { body, .. } = &mut stmt.kind {
                let taken = std::mem::take(body);
                *body = taken.into_iter().map(|stmt| split_function_decls(stmt, facts)).collect();
            }
            stmt
        }
        _ => stmt,
    }
}

/// Body-wide constructs that make a renamed binding observable or aliasable.
const BLOCKING_BODY_TEXT: &[&str] = &[
    "Closure {", "IncludeValue", "Include {", "IncludeOnceGuard", "Global {", "StaticVar",
    "RefAssign", "Reference(", "value_by_ref: true", "Goto(", "GotoLabel(", "Runtime(",
    "\"eval\"", "compact", "extract", "get_defined_vars", "parse_str", "GLOBALS", "func_get_arg",
    "NamedArg", "Yield", "__elephc_web",
];

/// Superglobals are visible in every scope, so a binding of one is never a local live range.
const SUPERGLOBALS: &[&str] = &[
    "GLOBALS", "_GET", "_POST", "_COOKIE", "_FILES", "_ENV", "_REQUEST", "_SERVER", "_SESSION",
];

/// Facts about the whole program that every scope's decision consults.
struct ProgramFacts {
    /// Top-level locals are globals: another function's `global $w` or `$GLOBALS` reads them.
    top_level_shared: bool,
    /// `extern global` names are C globals visible in every scope.
    extern_globals: Vec<String>,
}

/// Renames qualifying trailing `foreach` key bindings of one variable scope.
///
/// Only the KEY binding is split: PHP assigns the value first and the key second, and lowering
/// releases the earlier range after both bindings, which is exactly where the original key
/// assignment dropped it. Statement texts are rendered once, so the pass stays linear in the
/// body for any number of loops sharing a name.
fn split_body(body: Vec<Stmt>, params: &[String], top_level: bool, facts: &ProgramFacts) -> Vec<Stmt> {
    let texts: Vec<Option<String>> = body.iter()
        .map(|stmt| (!is_declaration(stmt)).then(|| format!("{stmt:?}")))
        .collect();
    if texts.iter().flatten().any(|text| BLOCKING_BODY_TEXT.iter().any(|blocked| text.contains(blocked))) {
        return body;
    }
    if top_level && facts.top_level_shared {
        return body;
    }
    let mut names: Vec<String> = body.iter().filter_map(|stmt| match &stmt.kind {
        StmtKind::Foreach { key_var: Some(key), value_var, value_by_ref: false, .. }
            if key != value_var => Some(key.clone()),
        _ => None,
    }).collect();
    names.sort();
    names.dedup();
    let mut body = body;
    for name in names {
        if params.contains(&name) || name == "this" || SUPERGLOBALS.contains(&name.as_str())
            || facts.extern_globals.contains(&name)
        {
            continue;
        }
        let quoted = format!("{name:?}");
        let variable = format!("Expr {{ kind: Variable({quoted})");
        let call_arg = [format!("args: [{variable}"), format!("}}, {variable}")];
        let mentions: Vec<bool> = texts.iter()
            .map(|text| text.as_ref().is_some_and(|text| text.contains(&quoted)))
            .collect();
        let Some(last) = mentions.iter().rposition(|mentioned| *mentioned) else { continue };
        // Only the LAST mention can be split, and only when an earlier range exists.
        let StmtKind::Foreach { key_var: Some(key), .. } = &body[last].kind else { continue };
        if key != &name || !mentions[..last].iter().any(|mentioned| *mentioned) {
            continue;
        }
        // An earlier range passed to a callee could leave a by-reference alias behind.
        if texts[..last].iter().flatten().any(|text| call_arg.iter().any(|arg| text.contains(arg))) {
            continue;
        }
        if let Some(renamed) = rename_trailing_foreach(&body[last], last, &name, &quoted) {
            body[last] = renamed;
        }
    }
    body
}

/// Excludes nested declarations, whose bodies are separate scopes, from the body-wide checks.
fn is_declaration(stmt: &Stmt) -> bool {
    matches!(stmt.kind, StmtKind::FunctionDecl { .. } | StmtKind::ClassDecl { .. }
        | StmtKind::EnumDecl { .. } | StmtKind::InterfaceDecl { .. } | StmtKind::TraitDecl { .. }
        | StmtKind::PackedClassDecl { .. } | StmtKind::ExternFunctionDecl { .. }
        | StmtKind::ExternClassDecl { .. } | StmtKind::ExternGlobalDecl { .. }
        | StmtKind::FunctionVariantGroup { .. } | StmtKind::FunctionVariantMark { .. })
}

/// Builds the loop with its key renamed, or `None` when the split is not provably complete.
fn rename_trailing_foreach(stmt: &Stmt, index: usize, name: &str, quoted: &str) -> Option<Stmt> {
    let StmtKind::Foreach { array, body: loop_body, .. } = &stmt.kind else { return None };
    if format!("{array:?}").contains(quoted) {
        return None;
    }
    let fresh = format!("{name}{WEB_SUFFIX}{index}");
    let mut pass = RenameVariable { from: name, to: &fresh };
    let renamed_body = walk_program(loop_body.clone(), &mut pass);
    let text = format!("{renamed_body:?}");
    // Every position the walker missed would still name the old variable, and a loop that never
    // reads its key has nothing to gain (and would change unused-variable diagnostics).
    if text.contains(quoted) || !text.contains(&format!("Variable({fresh:?})")) {
        return None;
    }
    let mut renamed = stmt.clone();
    let StmtKind::Foreach { key_var, body: new_body, .. } = &mut renamed.kind else { return None };
    *key_var = Some(fresh);
    *new_body = renamed_body;
    RELEASE_AT_ITERATION.with(|map| map.borrow_mut().insert(renamed.span, name.to_string()));
    Some(renamed)
}

/// Suffix of a split range's fresh name; diagnostics strip it back to the PHP name.
pub(crate) const WEB_SUFFIX: &str = "__elephc_web";

/// Returns the PHP-visible spelling of a local name, without a split range's suffix.
pub fn source_variable_name(name: &str) -> &str {
    match name.find(WEB_SUFFIX) {
        Some(at) if name[at + WEB_SUFFIX.len()..].bytes().all(|byte| byte.is_ascii_digit()) => &name[..at],
        _ => name,
    }
}

/// Rewrites every split range's fresh name in a diagnostic message back to its PHP name.
pub fn source_variable_message(message: &str) -> std::borrow::Cow<'_, str> {
    if !message.contains(WEB_SUFFIX) {
        return std::borrow::Cow::Borrowed(message);
    }
    let mut out = String::with_capacity(message.len());
    let mut rest = message;
    while let Some(at) = rest.find(WEB_SUFFIX) {
        out.push_str(&rest[..at]);
        let tail = &rest[at + WEB_SUFFIX.len()..];
        let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            out.push_str(WEB_SUFFIX);
        }
        rest = &tail[digits..];
    }
    out.push_str(rest);
    std::borrow::Cow::Owned(out)
}

thread_local! {
    /// Renamed loops, by statement span, and the earlier range's name each one retires.
    static RELEASE_AT_ITERATION: std::cell::RefCell<std::collections::HashMap<Span, String>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Returns the local an iteration of the renamed `foreach` at `span` must release, if any.
///
/// The original binding overwrote that local at the first iteration, which is when PHP drops
/// its previous value (and runs a destructor). Lowering releases the slot at the same point
/// without storing anything, so the earlier range keeps its own type.
pub fn released_at_iteration(span: Span) -> Option<String> {
    RELEASE_AT_ITERATION.with(|map| map.borrow().get(&span).cloned())
}

/// Renames every variable position naming `from` to `to`.
struct RenameVariable<'a> {
    from: &'a str,
    to: &'a str,
}

impl Pass for RenameVariable<'_> {
    fn transform_magic(&self, _span: Span, mc: MagicConstant) -> ExprKind {
        ExprKind::MagicConstant(mc)
    }

    fn transform_variable(&self, name: String) -> String {
        if name == self.from { self.to.to_string() } else { name }
    }
}
