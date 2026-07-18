//! Purpose:
//! Scans the typed program before emission to discover globals, constants, and static storage needs.
//! Seeds codegen context with symbols that later passes reference from generated assembly.
//!
//! Called from:
//! - `crate::codegen::generate()` before main and function emission
//!
//! Key details:
//! - The scan must mirror AST constructs that can allocate storage without evaluating program side effects.

use std::collections::{HashMap, HashSet};

use crate::codegen::platform::Platform;
use crate::parser::ast::{ExprKind, Program, Stmt, StmtKind};
use crate::types::array_constants::ARRAY_INT_CONSTANTS;
use crate::types::date_constants::{DATE_INT_CONSTANTS, DATE_STRING_CONSTANTS};
use crate::types::error_constants::ERROR_INT_CONSTANTS;
use crate::types::json_constants::JSON_INT_CONSTANTS;
use crate::types::php_runtime_constants::{
    PHP_RUNTIME_INT_CONSTANTS, PHP_RUNTIME_PLATFORM_CONSTANTS, PHP_SAPI_STR, PHP_VERSION_STR,
};
use crate::types::stream_constants::STREAM_INT_CONSTANTS;
use crate::types::locale_constants::LOCALE_INT_CONSTANTS;
use crate::types::preg_constants::{PCRE_VERSION_STR, PREG_INT_CONSTANTS};
use crate::types::string_constants::STRING_INT_CONSTANTS;
use crate::types::sort_constants::SORT_INT_CONSTANTS;
use crate::types::mbstring_constants::MBSTRING_INT_CONSTANTS;
use crate::types::filter_constants::FILTER_INT_CONSTANTS;
use crate::types::pcntl_constants::{PCNTL_INT_CONSTANTS, PCNTL_PLATFORM_SIGNALS};
use crate::types::upload_constants::UPLOAD_ERR_INT_CONSTANTS;
use crate::types::url_constants::URL_INT_CONSTANTS;
use crate::types::tokenizer_constants::TOKENIZER_INT_CONSTANTS;
use crate::types::xml_constants::XML_INT_CONSTANTS;
use crate::types::{PhpType, TypeEnv};

use super::context::{Context, TRY_HANDLER_SLOT_SIZE};

/// Seeds the constant map with built-in PHP constants and user-defined constants.
///
/// Built-in constants include platform-specific values (e.g., `FNM_*` flags differ
/// between macOS and Linux), `PATHINFO_*` bitmask values, stream handles (`STDIN`/`STDOUT`/`STDERR`),
/// `LOCK_*` values, array callback-mode constants, `JSON_*` integer constants, and
/// `PREG_*` integer constants. User constants come from `const` declarations and
/// `define()` calls discovered by `collect_constant_decls`.
pub(crate) fn collect_constants(
    program: &Program,
    target_platform: Platform,
) -> HashMap<String, (ExprKind, PhpType)> {
    let mut constants = HashMap::new();
    constants.insert(
        "PHP_OS".to_string(),
        (
            ExprKind::StringLiteral(target_platform.php_os_name().to_string()),
            PhpType::Str,
        ),
    );
    constants.insert(
        "PATHINFO_DIRNAME".to_string(),
        (ExprKind::IntLiteral(1), PhpType::Int),
    );
    constants.insert(
        "PATHINFO_BASENAME".to_string(),
        (ExprKind::IntLiteral(2), PhpType::Int),
    );
    constants.insert(
        "PATHINFO_EXTENSION".to_string(),
        (ExprKind::IntLiteral(4), PhpType::Int),
    );
    constants.insert(
        "PATHINFO_FILENAME".to_string(),
        (ExprKind::IntLiteral(8), PhpType::Int),
    );
    constants.insert(
        "PATHINFO_ALL".to_string(),
        (ExprKind::IntLiteral(15), PhpType::Int),
    );
    let (fnm_noescape, fnm_pathname) = match target_platform {
        Platform::MacOS => (1, 2),
        Platform::Linux => (2, 1),
    };
    constants.insert(
        "FNM_NOESCAPE".to_string(),
        (ExprKind::IntLiteral(fnm_noescape), PhpType::Int),
    );
    constants.insert(
        "FNM_PATHNAME".to_string(),
        (ExprKind::IntLiteral(fnm_pathname), PhpType::Int),
    );
    constants.insert(
        "FNM_PERIOD".to_string(),
        (ExprKind::IntLiteral(4), PhpType::Int),
    );
    constants.insert(
        "FNM_CASEFOLD".to_string(),
        (ExprKind::IntLiteral(16), PhpType::Int),
    );
    constants.insert(
        "STDIN".to_string(),
        (ExprKind::IntLiteral(0), PhpType::stream_resource()),
    );
    constants.insert(
        "STDOUT".to_string(),
        (ExprKind::IntLiteral(1), PhpType::stream_resource()),
    );
    constants.insert(
        "STDERR".to_string(),
        (ExprKind::IntLiteral(2), PhpType::stream_resource()),
    );
    constants.insert(
        "LOCK_SH".to_string(),
        (ExprKind::IntLiteral(1), PhpType::Int),
    );
    constants.insert(
        "LOCK_EX".to_string(),
        (ExprKind::IntLiteral(2), PhpType::Int),
    );
    constants.insert(
        "LOCK_UN".to_string(),
        (ExprKind::IntLiteral(3), PhpType::Int),
    );
    constants.insert(
        "LOCK_NB".to_string(),
        (ExprKind::IntLiteral(4), PhpType::Int),
    );
    for (name, value) in ARRAY_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in JSON_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in STREAM_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in PREG_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in DATE_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in ERROR_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in PHP_RUNTIME_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in LOCALE_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in STRING_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in SORT_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in MBSTRING_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in FILTER_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in PCNTL_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in UPLOAD_ERR_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in URL_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in TOKENIZER_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in XML_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    // Platform-conditional user signals (SIGUSR1/SIGUSR2): select the value from
    // the COMPILE target (not `cfg(target_os)`, since elephc cross-compiles).
    // macOS: SIGUSR1=30, SIGUSR2=31. Linux (x86_64 and aarch64): SIGUSR1=10,
    // SIGUSR2=12. Mirrors the fnmatch `match target_platform` pattern above.
    for (name, macos_value, linux_value) in PCNTL_PLATFORM_SIGNALS {
        let value = match target_platform {
            Platform::MacOS => macos_value,
            Platform::Linux => linux_value,
        };
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    // Platform-conditional runtime constants (PHP_MAXPATHLEN): same
    // compile-target selection as the pcntl user signals above.
    for (name, macos_value, linux_value) in PHP_RUNTIME_PLATFORM_CONSTANTS {
        let value = match target_platform {
            Platform::MacOS => macos_value,
            Platform::Linux => linux_value,
        };
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    constants.insert(
        "PCRE_VERSION".to_string(),
        (ExprKind::StringLiteral(PCRE_VERSION_STR.to_string()), PhpType::Str),
    );
    constants.insert(
        "PHP_SAPI".to_string(),
        (ExprKind::StringLiteral(PHP_SAPI_STR.to_string()), PhpType::Str),
    );
    constants.insert(
        "PHP_VERSION".to_string(),
        (ExprKind::StringLiteral(PHP_VERSION_STR.to_string()), PhpType::Str),
    );
    constants.insert(
        "PHP_OS_FAMILY".to_string(),
        (
            ExprKind::StringLiteral(target_platform.php_os_family().to_string()),
            PhpType::Str,
        ),
    );
    // DATE_* format-string constants (DATE_ATOM, DATE_RFC3339, ...).
    for (name, value) in DATE_STRING_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::StringLiteral((*value).to_string()), PhpType::Str),
        );
    }
    collect_constant_decls(program, &mut constants);
    constants
}

/// Recursively scans statements for user-defined constant declarations.
///
/// Visits `const` declarations and `define()` function calls, inserting each
/// constant's name, expression, and inferred type into `constants`. Recurses into
/// every body-bearing statement kind — function/method bodies, control-flow bodies
/// (if/loops/try/switch), namespace blocks, and include/synthetic bodies — so an
/// in-function or in-method literal `define()` is discovered too.
///
/// AOT approximation: a `define()` found inside a function/method or a conditional
/// branch is registered as program-wide defined regardless of whether or when the
/// enclosing code actually runs. This mirrors the pre-existing approximation for a
/// top-level `define()` and is strictly better than the previous behaviour, where an
/// in-function `define()` was invisible and reads SIGSEGV'd. `.or_insert` keeps the
/// first-found definition, and callers walk top-level statements first, so a
/// top-level define consistently shadows an in-function one. A genuinely
/// never-defined constant read remains out of scope (the `LoadGlobal` fallback should
/// ideally throw `\Error` rather than SIGSEGV — separate hardening).
fn collect_constant_decls(
    stmts: &[Stmt],
    constants: &mut HashMap<String, (ExprKind, PhpType)>,
) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::ConstDecl { name, value } => {
                constants
                    .entry(name.clone())
                    .or_insert((value.kind.clone(), constant_expr_type(&value.kind)));
            }
            StmtKind::ExprStmt(expr) => {
                register_define_from_expr(&expr.kind, constants);
            }
            StmtKind::Return(Some(expr)) => {
                register_define_from_expr(&expr.kind, constants);
            }
            StmtKind::FunctionDecl { body, .. }
            | StmtKind::IncludeOnceGuard { body, .. }
            | StmtKind::NamespaceBlock { body, .. } => {
                collect_constant_decls(body, constants);
            }
            StmtKind::Synthetic(body) => {
                collect_constant_decls(body, constants);
            }
            StmtKind::ClassDecl { methods, .. }
            | StmtKind::TraitDecl { methods, .. }
            | StmtKind::EnumDecl { methods, .. } => {
                for method in methods {
                    collect_constant_decls(&method.body, constants);
                }
            }
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                collect_constant_decls(then_body, constants);
                for (_, body) in elseif_clauses {
                    collect_constant_decls(body, constants);
                }
                if let Some(body) = else_body {
                    collect_constant_decls(body, constants);
                }
            }
            StmtKind::While { body, .. }
            | StmtKind::DoWhile { body, .. }
            | StmtKind::For { body, .. }
            | StmtKind::Foreach { body, .. } => collect_constant_decls(body, constants),
            StmtKind::Try {
                try_body,
                catches,
                finally_body,
            } => {
                collect_constant_decls(try_body, constants);
                for catch_clause in catches {
                    collect_constant_decls(&catch_clause.body, constants);
                }
                if let Some(body) = finally_body {
                    collect_constant_decls(body, constants);
                }
            }
            StmtKind::Switch { cases, default, .. } => {
                for (_, body) in cases {
                    collect_constant_decls(body, constants);
                }
                if let Some(body) = default {
                    collect_constant_decls(body, constants);
                }
            }
            _ => {}
        }
    }
}

/// Registers a `define("NAME", <scalar-literal>)` call encountered as a statement
/// expression (a bare `define(...)` or `return define(...)`).
///
/// Only inserts when arg0 is a `StringLiteral` and arg1 is a scalar literal; keeps
/// `.or_insert` semantics so the first-found definition wins. Mirrors the top-level
/// `define()` handling so an in-function `define()` becomes program-wide visible.
fn register_define_from_expr(
    kind: &ExprKind,
    constants: &mut HashMap<String, (ExprKind, PhpType)>,
) {
    if let ExprKind::FunctionCall { name, args } = kind {
        if name.as_str() == "define" && args.len() == 2 {
            if let ExprKind::StringLiteral(const_name) = &args[0].kind {
                if is_scalar_literal(&args[1].kind) {
                    constants
                        .entry(const_name.clone())
                        .or_insert((args[1].kind.clone(), constant_expr_type(&args[1].kind)));
                }
            }
        }
    }
}

/// Reports whether an expression is a scalar literal usable as a constant value.
///
/// Only `Int`/`Float`/`Bool`/`Str`/`Null` literals qualify; any other expression
/// (variables, calls, operations) is rejected so a non-constant `define()` argument
/// is not prescanned as a compile-time constant.
fn is_scalar_literal(kind: &ExprKind) -> bool {
    matches!(
        kind,
        ExprKind::IntLiteral(_)
            | ExprKind::FloatLiteral(_)
            | ExprKind::StringLiteral(_)
            | ExprKind::BoolLiteral(_)
            | ExprKind::Null
    )
}

/// Infers the `PhpType` for a constant expression from its `ExprKind` variant.
///
/// Returns `PhpType::Int` as a fallback for unsupported expression kinds.
/// Does not evaluate the expression; only maps literal variants to their types.
fn constant_expr_type(kind: &ExprKind) -> PhpType {
    match kind {
        ExprKind::IntLiteral(_) => PhpType::Int,
        ExprKind::FloatLiteral(_) => PhpType::Float,
        ExprKind::StringLiteral(_) => PhpType::Str,
        ExprKind::BoolLiteral(_) => PhpType::Bool,
        ExprKind::Null => PhpType::Void,
        _ => PhpType::Int,
    }
}

/// Collects the names of all PHP `global` variables declared inside user functions.
///
/// Scans every function body in the program (but not the top level) and gathers
/// variable names from `global` declarations. Returns a set of global variable
/// names used to seed the codegen context.
pub(super) fn collect_global_var_names(program: &Program) -> HashSet<String> {
    let mut names = HashSet::new();
    for stmt in program {
        if let StmtKind::FunctionDecl { body, .. } = &stmt.kind {
            collect_global_vars_in_body(body, &mut names);
        }
    }
    names
}

/// Recursively gathers `global` variable names from a statement list.
///
/// Helper for `collect_global_var_names`. Descends into control-flow bodies
/// (if/while/for/foreach/try/switch) but ignores top-level declarations.
fn collect_global_vars_in_body(stmts: &[Stmt], names: &mut HashSet<String>) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::Global { vars } => {
                for v in vars {
                    names.insert(v.clone());
                }
            }
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                collect_global_vars_in_body(then_body, names);
                for (_, body) in elseif_clauses {
                    collect_global_vars_in_body(body, names);
                }
                if let Some(body) = else_body {
                    collect_global_vars_in_body(body, names);
                }
            }
            StmtKind::While { body, .. }
            | StmtKind::DoWhile { body, .. }
            | StmtKind::For { body, .. }
            | StmtKind::Foreach { body, .. }
            | StmtKind::IncludeOnceGuard { body, .. } => collect_global_vars_in_body(body, names),
            StmtKind::Try {
                try_body,
                catches,
                finally_body,
            } => {
                collect_global_vars_in_body(try_body, names);
                for catch_clause in catches {
                    collect_global_vars_in_body(&catch_clause.body, names);
                }
                if let Some(body) = finally_body {
                    collect_global_vars_in_body(body, names);
                }
            }
            StmtKind::Switch { cases, default, .. } => {
                for (_, body) in cases {
                    collect_global_vars_in_body(body, names);
                }
                if let Some(body) = default {
                    collect_global_vars_in_body(body, names);
                }
            }
            _ => {}
        }
    }
}

/// Collects all static variables declared inside user functions, keyed by `(function_name, var_name)`.
///
/// Scans every function body in the program and registers `static` variable declarations
/// with their inferred type (from the initializer expression). The resulting map
/// seeds the codegen context to emit static storage.
pub(super) fn collect_static_vars(
    program: &Program,
    global_env: &TypeEnv,
) -> HashMap<(String, String), PhpType> {
    let mut statics = HashMap::new();
    for stmt in program {
        if let StmtKind::FunctionDecl { name, body, .. } = &stmt.kind {
            collect_static_vars_in_body(name, body, &mut statics, global_env);
        }
    }
    statics
}

/// Recursively gathers `static` variable declarations from a function body.
///
/// Helper for `collect_static_vars`. Descends into control-flow bodies and
/// registers each `static var` with its `(func_name, var_name)` key and inferred type.
/// The `global_env` parameter is unused but accepted for API compatibility.
fn collect_static_vars_in_body(
    func_name: &str,
    stmts: &[Stmt],
    statics: &mut HashMap<(String, String), PhpType>,
    global_env: &TypeEnv,
) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::StaticVar { name, init } => {
                let ty = match &init.kind {
                    ExprKind::IntLiteral(_) => PhpType::Int,
                    ExprKind::FloatLiteral(_) => PhpType::Float,
                    ExprKind::StringLiteral(_) => PhpType::Str,
                    ExprKind::BoolLiteral(_) => PhpType::Bool,
                    _ => PhpType::Int,
                };
                statics.insert((func_name.to_string(), name.clone()), ty);
            }
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                collect_static_vars_in_body(func_name, then_body, statics, global_env);
                for (_, body) in elseif_clauses {
                    collect_static_vars_in_body(func_name, body, statics, global_env);
                }
                if let Some(body) = else_body {
                    collect_static_vars_in_body(func_name, body, statics, global_env);
                }
            }
            StmtKind::While { body, .. }
            | StmtKind::DoWhile { body, .. }
            | StmtKind::For { body, .. }
            | StmtKind::Foreach { body, .. }
            | StmtKind::IncludeOnceGuard { body, .. } => {
                collect_static_vars_in_body(func_name, body, statics, global_env);
            }
            StmtKind::Try {
                try_body,
                catches,
                finally_body,
            } => {
                collect_static_vars_in_body(func_name, try_body, statics, global_env);
                for catch_clause in catches {
                    collect_static_vars_in_body(
                        func_name,
                        &catch_clause.body,
                        statics,
                        global_env,
                    );
                }
                if let Some(body) = finally_body {
                    collect_static_vars_in_body(func_name, body, statics, global_env);
                }
            }
            StmtKind::Switch { cases, default, .. } => {
                for (_, body) in cases {
                    collect_static_vars_in_body(func_name, body, statics, global_env);
                }
                if let Some(body) = default {
                    collect_static_vars_in_body(func_name, body, statics, global_env);
                }
            }
            _ => {}
        }
    }
    let _ = global_env;
}

/// Pre-allocates hidden stack slots for every `try` block in the top-level program.
///
/// Each `try` block requires a dedicated slot to store the handler pointer during
/// unwinding. This scans the main statement list (not function bodies) recursively,
/// allocating a slot and pushing its offset for each `try` encountered. Nested
/// try/catch/finally structures are handled recursively.
pub(super) fn collect_main_try_slots(stmts: &[Stmt], ctx: &mut Context) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::Try {
                try_body,
                catches,
                finally_body,
            } => {
                let slot_offset = ctx.alloc_hidden_slot(TRY_HANDLER_SLOT_SIZE);
                ctx.try_slot_offsets.push(slot_offset);
                collect_main_try_slots(try_body, ctx);
                for catch_clause in catches {
                    collect_main_try_slots(&catch_clause.body, ctx);
                }
                if let Some(body) = finally_body {
                    collect_main_try_slots(body, ctx);
                }
            }
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                collect_main_try_slots(then_body, ctx);
                for (_, body) in elseif_clauses {
                    collect_main_try_slots(body, ctx);
                }
                if let Some(body) = else_body {
                    collect_main_try_slots(body, ctx);
                }
            }
            StmtKind::While { body, .. }
            | StmtKind::DoWhile { body, .. }
            | StmtKind::Foreach { body, .. }
            | StmtKind::IncludeOnceGuard { body, .. } => collect_main_try_slots(body, ctx),
            StmtKind::For {
                init, update, body, ..
            } => {
                if let Some(s) = init {
                    collect_main_try_slots(&[*s.clone()], ctx);
                }
                if let Some(s) = update {
                    collect_main_try_slots(&[*s.clone()], ctx);
                }
                collect_main_try_slots(body, ctx);
            }
            StmtKind::Switch { cases, default, .. } => {
                for (_, body) in cases {
                    collect_main_try_slots(body, ctx);
                }
                if let Some(body) = default {
                    collect_main_try_slots(body, ctx);
                }
            }
            StmtKind::FunctionDecl { .. }
            | StmtKind::ClassDecl { .. }
            | StmtKind::InterfaceDecl { .. }
            | StmtKind::TraitDecl { .. } => {}
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Implements the `int_constant` operation for this module.
    fn int_constant(constants: &HashMap<String, (ExprKind, PhpType)>, name: &str) -> i64 {
        match &constants[name].0 {
            ExprKind::IntLiteral(value) => *value,
            _ => panic!("{name} is not an integer constant"),
        }
    }

    /// Verifies fnmatch constants follow target platform.
    #[test]
    fn test_fnmatch_constants_follow_target_platform() {
        let mac = collect_constants(&vec![], Platform::MacOS);
        assert_eq!(int_constant(&mac, "FNM_NOESCAPE"), 1);
        assert_eq!(int_constant(&mac, "FNM_PATHNAME"), 2);
        assert_eq!(int_constant(&mac, "FNM_PERIOD"), 4);
        assert_eq!(int_constant(&mac, "FNM_CASEFOLD"), 16);

        let linux = collect_constants(&vec![], Platform::Linux);
        assert_eq!(int_constant(&linux, "FNM_NOESCAPE"), 2);
        assert_eq!(int_constant(&linux, "FNM_PATHNAME"), 1);
        assert_eq!(int_constant(&linux, "FNM_PERIOD"), 4);
        assert_eq!(int_constant(&linux, "FNM_CASEFOLD"), 16);
    }
}
