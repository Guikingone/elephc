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

/// Verifies no injected compiler prelude calls a PHP-visible extension builtin.
///
/// `--strict-php` hides extension builtins at the catalog level with no notion
/// of code origin, so a prelude calling one (instead of its `internal: true`
/// `__elephc_*` alias) would break strict-mode compiles of programs that
/// trigger that prelude's injection. Scans every prelude PHP source for
/// `<name>(` call sites; bare mentions inside comments are tolerated.
#[test]
fn preludes_never_call_php_visible_extension_builtins() {
    let extension_names = shared_php_visible_extension_contracts()
        .map(|contract| contract.name.to_string())
        .collect::<Vec<_>>();

    let prelude_sources: &[(&str, &str)] = &[
        ("pdo_prelude", crate::pdo_prelude::PDO_PRELUDE_SRC),
        ("tz_prelude", crate::tz_prelude::TZ_PRELUDE_SRC),
        ("list_id_prelude", crate::list_id_prelude::LIST_ID_PRELUDE_TEMPLATE),
        ("var_export_prelude", crate::var_export_prelude::VAR_EXPORT_PRELUDE_SRC),
        ("image_prelude", crate::image_prelude::IMAGE_PRELUDE_SRC),
        ("web_prelude", crate::web_prelude::WEB_PRELUDE_SRC),
        ("web_prelude(wrap)", crate::web_prelude::WEB_WRAP_SRC),
    ];

    let mut violations: Vec<String> = Vec::new();
    for (prelude, source) in prelude_sources {
        for name in &extension_names {
            if source_calls_function(source, name) {
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

/// Returns true when `source` contains a plain function-call site `name(`.
///
/// A match is a call site only when the preceding character is not part of a
/// longer identifier (`elephc_pdo_column_data_ptr(`), a variable (`$ptr(`), or
/// a method/static access (`->ptr(`, `::ptr(`), so extern helpers whose names
/// merely end with a builtin name do not count.
fn source_calls_function(source: &str, name: &str) -> bool {
    let needle = format!("{name}(");
    source.match_indices(&needle).any(|(index, _)| {
        match source[..index].chars().next_back() {
            None => true,
            Some(prev) => {
                !prev.is_ascii_alphanumeric() && !matches!(prev, '_' | '$' | '>' | ':')
            }
        }
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
