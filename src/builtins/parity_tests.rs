//! Purpose:
//! Structural parity gates for registry visibility, extension classification,
//! strict-PHP behavior, and compiler prelude usage.
//!
//! Called from:
//! - `cargo test` through Rust's test harness (unit test module).
//!
//! Key details:
//! - Registry signatures are authoritative; no parallel name-based golden table exists.
//! - Extension and internal visibility derive from shared contracts.

use crate::builtins::registry;
use elephc_builtin_contract::{
    aot_support, BackendImplementation, BackendSupport, BuiltinContract,
};

/// Returns the PHP-visible extension builtins a prelude must never call directly.
fn php_visible_extension_builtins() -> Vec<String> {
    let mut names: Vec<String> = vec!["buffer_new".to_string()];
    for name in registry::names() {
        let def = registry::lookup(name).expect("names() yields registered builtins");
        if def.spec.extension && !def.spec.internal {
            names.push(def.name.to_string());
        }
    }
    names
}

/// Verifies no injected compiler prelude calls a PHP-visible extension builtin.
///
/// `--strict-php` hides extension builtins at the catalog level with no notion of code origin,
/// so a prelude calling one (instead of its `internal: true` `__elephc_*` alias) would break
/// strict-mode compiles of programs that trigger that prelude's injection.
///
/// THIS USED TO BE HALF A GATE. Every prelude was PHP text, and the audit was a `grep` for
/// `name(` that tolerated bare mentions in comments and had to reason about the character before
/// the match to tell a call from `elephc_pdo_column_data_ptr(` or `->ptr(`. Now that every
/// prelude builds its declarations in Rust, the audit just reads the call sites off the AST —
/// and `called_function_names` panics on any node it does not model, so a prelude that grows a
/// construct this audit cannot see fails loudly instead of silently leaving the net.
#[test]
fn preludes_built_in_rust_never_call_php_visible_extension_builtins() {
    let extension_names = php_visible_extension_builtins();

    let mut built: Vec<(&str, crate::parser::ast::Program)> = vec![
        ("hash_prelude", crate::hash_prelude::hash_declarations()),
        ("tz_prelude", crate::tz_prelude::tz_declarations()),
        (
            "var_export_prelude",
            crate::var_export_prelude::var_export_declarations(),
        ),
        (
            "list_id_prelude",
            crate::list_id_prelude::list_id_declarations(),
        ),
        (
            "pdo_prelude",
            crate::pdo_prelude::build::pdo_declarations(
                crate::php_version::PhpVersion::default(),
                crate::pdo_prelude::OptionalDrivers::from_build_environment(),
            ),
        ),
        ("image_prelude", crate::image_prelude::image_declarations()),
        (
            "version_prelude",
            crate::version_prelude::version_declarations(
                &["zend_version", "php_sapi_name", "ini_restore"],
                crate::web_prelude::PhpVersion::Php85,
            ),
        ),
        (
            "opcache_prelude(env)",
            crate::opcache_prelude::env_override_declarations(),
        ),
        (
            "opcache_prelude(ini)",
            crate::opcache_prelude::ini_helper_declarations(
                crate::web_prelude::PhpVersion::Php85,
                &[],
            ),
        ),
        (
            "opcache_prelude(state)",
            crate::opcache_prelude::build::state_helper_decls(),
        ),
        (
            "web_prelude",
            crate::web_prelude::build::web_declarations(
                crate::web_prelude::PhpVersion::Php85,
                &[],
            ),
        ),
        ("web_prelude(wrap)", vec![crate::web_prelude::web_wrap_stmt()]),
    ];
    // The mysqli prelude is a second bridge surface whose PHP fragments are
    // parsed (not built as Rust AST like the others), so parse each fragment and
    // scan it too — the gate must cover mysqli's `__elephc_*` internal-alias
    // discipline. The shared `elephc_pdo` extern block declares only symbols and
    // calls nothing, and the PDO build above already carries it, so it needs no
    // separate entry.
    for &(name, src) in crate::mysqli_prelude::fragment_sources() {
        let tokens = crate::lexer::tokenize(src).expect("mysqli fragment must tokenize");
        let program =
            crate::parser::parse_internal(&tokens).expect("mysqli fragment must parse");
        built.push((name, program));
    }

    let mut violations: Vec<String> = Vec::new();
    for (prelude, program) in &built {
        let called = crate::synthetic_class::called_function_names(program);
        for name in &extension_names {
            if called.iter().any(|call| call.eq_ignore_ascii_case(name)) {
                violations.push(format!("{prelude} calls {name}()"));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "preludes must call `__elephc_*` internal aliases, not PHP-visible extension builtins:\n{}",
        violations.join("\n"),
    );
}

/// Returns shared extension contracts visible through an AOT PHP call surface.
fn shared_php_visible_extension_contracts(
) -> impl Iterator<Item = &'static BuiltinContract> {
    elephc_builtin_contract::contracts()
        .iter()
        .filter(|contract| contract.extension && !contract.internal)
        .filter(|contract| {
            matches!(
                aot_support(contract),
                BackendSupport::Implemented(
                    BackendImplementation::Registry | BackendImplementation::DedicatedSyntax
                )
            )
        })
}

/// Verifies the AOT registry exposes exactly the shared registry-route extensions.
#[test]
fn extension_builtin_set_matches_shared_contracts() {
    let mut tagged: Vec<&str> = Vec::new();
    for name in registry::names() {
        let def = registry::lookup(name).expect("names() yields registered builtins");
        if def.spec.extension && !def.spec.internal {
            tagged.push(def.name);
        }
    }
    tagged.sort_unstable();
    let expected = shared_php_visible_extension_contracts()
        .filter(|contract| {
            matches!(
                aot_support(contract),
                BackendSupport::Implemented(BackendImplementation::Registry)
            )
        })
        .map(|contract| contract.name)
        .collect::<Vec<_>>();
    assert_eq!(tagged, expected, "AOT extension contract join drifted");
}
