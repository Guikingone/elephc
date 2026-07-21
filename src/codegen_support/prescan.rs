//! Purpose:
//! Scans the program for compile-time constants used by lowering and codegen.
//! Seeds the constant map with builtin and user-defined constant values.
//!
//! Called from:
//! - `crate::ir_lower::program` through `crate::codegen::collect_constants`.
//!
//! Key details:
//! - The scan must not evaluate PHP side effects; it only recognizes declarations and literal `define()` calls.

use std::collections::HashMap;

use crate::codegen_support::platform::Platform;
use crate::parser::ast::{ExprKind, Program, Stmt, StmtKind};
use crate::types::array_constants::ARRAY_INT_CONSTANTS;
use crate::types::date_constants::DATE_INT_CONSTANTS;
use crate::types::ent_constants::ENT_INT_CONSTANTS;
use crate::types::error_constants::ERROR_LEVEL_CONSTANTS;
use crate::types::json_constants::JSON_INT_CONSTANTS;
use crate::types::filter_constants::FILTER_INT_CONSTANTS;
use crate::types::locale_constants::LOCALE_INT_CONSTANTS;
use crate::types::mbstring_constants::MBSTRING_INT_CONSTANTS;
use crate::types::pcntl_constants::{PCNTL_INT_CONSTANTS, PCNTL_PLATFORM_SIGNALS};
use crate::types::php_runtime_constants::{
    PHP_RUNTIME_INT_CONSTANTS, PHP_RUNTIME_PLATFORM_CONSTANTS, PHP_SAPI_STR, PHP_VERSION_STR,
};
use crate::types::preg_constants::{PCRE_VERSION_STR, PREG_INT_CONSTANTS};
use crate::types::session_constants::SESSION_INT_CONSTANTS;
use crate::types::sort_constants::SORT_INT_CONSTANTS;
use crate::types::stream_constants::{GLOB_PLATFORM_CONSTANTS, STREAM_INT_CONSTANTS};
use crate::types::string_constants::STRING_INT_CONSTANTS;
use crate::types::tokenizer_constants::TOKENIZER_INT_CONSTANTS;
use crate::types::upload_constants::UPLOAD_ERR_INT_CONSTANTS;
use crate::types::url_constants::URL_INT_CONSTANTS;
use crate::types::xml_constants::XML_INT_CONSTANTS;
use crate::types::date_constants::DATE_STRING_CONSTANTS;
use crate::types::PhpType;

/// Seeds the constant map with built-in PHP constants and user-defined constants.
///
/// Built-in constants include platform-specific values (e.g., `FNM_*` flags differ
/// between macOS and Linux), `PATHINFO_*` bitmask values, `ENT_*` HTML-escaping flags,
/// stream handles (`STDIN`/`STDOUT`/`STDERR`), `LOCK_*` values, array callback-mode
/// constants, `JSON_*` integer constants, and `PREG_*` integer constants. User constants
/// come from `const` declarations and `define()` calls discovered by `collect_constant_decls`.
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
        "SID".to_string(),
        (ExprKind::StringLiteral(String::new()), PhpType::Str),
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
    for (name, value) in ENT_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    let (fnm_noescape, fnm_pathname) = match target_platform {
        Platform::MacOS => (1, 2),
        Platform::Linux => (2, 1),
        Platform::Windows => panic!("Windows target is not yet supported (see issue #379)"),
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
    for (name, value) in SESSION_INT_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    for (name, value) in ERROR_LEVEL_CONSTANTS {
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    // Lexer-tokenized numeric / math constants (also reachable via `use const` aliases).
    constants.insert(
        "PHP_INT_MAX".to_string(),
        (ExprKind::IntLiteral(i64::MAX), PhpType::Int),
    );
    constants.insert(
        "PHP_INT_MIN".to_string(),
        (ExprKind::IntLiteral(i64::MIN), PhpType::Int),
    );
    constants.insert(
        "PHP_FLOAT_MAX".to_string(),
        (ExprKind::FloatLiteral(f64::MAX), PhpType::Float),
    );
    constants.insert(
        "PHP_FLOAT_MIN".to_string(),
        (ExprKind::FloatLiteral(f64::MIN_POSITIVE), PhpType::Float),
    );
    constants.insert(
        "PHP_FLOAT_EPSILON".to_string(),
        (ExprKind::FloatLiteral(f64::EPSILON), PhpType::Float),
    );
    constants.insert(
        "INF".to_string(),
        (ExprKind::FloatLiteral(f64::INFINITY), PhpType::Float),
    );
    constants.insert(
        "NAN".to_string(),
        (ExprKind::FloatLiteral(f64::NAN), PhpType::Float),
    );
    constants.insert(
        "M_PI".to_string(),
        (ExprKind::FloatLiteral(std::f64::consts::PI), PhpType::Float),
    );
    constants.insert(
        "M_E".to_string(),
        (ExprKind::FloatLiteral(std::f64::consts::E), PhpType::Float),
    );
    constants.insert(
        "M_SQRT2".to_string(),
        (
            ExprKind::FloatLiteral(std::f64::consts::SQRT_2),
            PhpType::Float,
        ),
    );
    constants.insert(
        "M_PI_2".to_string(),
        (
            ExprKind::FloatLiteral(std::f64::consts::FRAC_PI_2),
            PhpType::Float,
        ),
    );
    constants.insert(
        "M_PI_4".to_string(),
        (
            ExprKind::FloatLiteral(std::f64::consts::FRAC_PI_4),
            PhpType::Float,
        ),
    );
    constants.insert(
        "M_LOG2E".to_string(),
        (
            ExprKind::FloatLiteral(std::f64::consts::LOG2_E),
            PhpType::Float,
        ),
    );
    constants.insert(
        "M_LOG10E".to_string(),
        (
            ExprKind::FloatLiteral(std::f64::consts::LOG10_E),
            PhpType::Float,
        ),
    );
    constants.insert(
        "PHP_EOL".to_string(),
        (ExprKind::StringLiteral("\n".to_string()), PhpType::Str),
    );
    constants.insert(
        "DIRECTORY_SEPARATOR".to_string(),
        (
            ExprKind::StringLiteral(std::path::MAIN_SEPARATOR.to_string()),
            PhpType::Str,
        ),
    );
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
            Platform::Windows => panic!("Windows target is not yet supported (see issue #379)"),
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
            Platform::Windows => panic!("Windows target is not yet supported (see issue #379)"),
        };
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    // Platform-conditional glob() bit flags (GLOB_MARK/NOSORT/BRACE/...): BSD
    // (macOS) and glibc (Linux) assign different bit positions to the same flag
    // names — same compile-target selection as the constants above.
    for (name, macos_value, linux_value) in GLOB_PLATFORM_CONSTANTS {
        let value = match target_platform {
            Platform::MacOS => macos_value,
            Platform::Linux => linux_value,
            Platform::Windows => panic!("Windows target is not yet supported (see issue #379)"),
        };
        constants.insert(
            (*name).to_string(),
            (ExprKind::IntLiteral(*value), PhpType::Int),
        );
    }
    constants.insert(
        "PCRE_VERSION".to_string(),
        (
            ExprKind::StringLiteral(PCRE_VERSION_STR.to_string()),
            PhpType::Str,
        ),
    );
    constants.insert(
        "PHP_SAPI".to_string(),
        (
            ExprKind::StringLiteral(PHP_SAPI_STR.to_string()),
            PhpType::Str,
        ),
    );
    constants.insert(
        "PHP_VERSION".to_string(),
        (
            ExprKind::StringLiteral(PHP_VERSION_STR.to_string()),
            PhpType::Str,
        ),
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
/// constant's name, expression, and inferred type into `constants`. Skips nested
/// functions/classes; only processes statement bodies at the top level and within
/// `IncludeOnceGuard` or synthetic bodies.
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

/// Registers a literal `define('NAME', <scalar literal>)` call found in expression
/// position. AOT approximation: the constant is registered program-wide regardless of
/// whether or when the enclosing code actually runs; `.or_insert` keeps the first-found
/// (top-level statements are walked first, so a top-level define shadows an in-function
/// one). Non-literal values are skipped — they cannot be folded at compile time.
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

/// Returns true for expression kinds that are replayable scalar literals (int, float,
/// string, bool, null) — the only `define()` values the prescan can register statically.
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
