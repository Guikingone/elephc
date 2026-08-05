//! Purpose:
//! Detects OPcache usage and injects the selected prelude functions.
//!
//! Called from:
//! - The OPcache prelude facade and sibling rendering modules.
//!
//! Key details:
//! - Per-function pay-for-use and user-declaration precedence remain authoritative.

#[allow(unused_imports)]
use super::*;

/// Prepends the OPcache prelude functions (`opcache_get_configuration`, `opcache_reset`,
/// `opcache_get_status`) each when the program references it and does not declare its
/// own, so unrelated binaries pay nothing and a user definition is not clobbered. `web`
/// is the compiler SAPI flag that selects the baked enabled/disabled state for
/// `opcache_reset` and `opcache_get_status` (a disabled cache makes `opcache_get_status`
/// return `false`, matching reference `php script.php`). The prelude
/// is hoisted function declarations only, so prepending does not change top-level
/// execution order. The rendered source is static data, so a tokenize/parse failure is
/// a compiler bug and panics rather than degrading silently.
///
/// `manifest` is the compile-time OPcache script manifest (see `ScriptEntry`). It is baked into
/// `opcache_get_status` (the `scripts` map and cached-script counts), `opcache_is_script_cached`,
/// and `opcache_compile_file`. An empty manifest renders valid PHP (empty `scripts` map, `false`
/// membership).
///
/// At THIS point the manifest is necessarily a PLACEHOLDER: the autoloaded file set does not
/// exist until `autoload::run`, which runs after name resolution — but the declarations must
/// exist BEFORE name resolution or a namespaced caller would not resolve to them. The returned
/// [`ManifestBakeSites`] records which manifest-dependent functions were injected so
/// [`bake_manifest`] can re-render exactly those against the complete manifest. See
/// [`bake_manifest`] for the full argument and the soundness of the substitution.
///
/// `entry_path` is the canonicalized entry script (`canonical_entry_path`), used ONLY for the
/// `opcache.restrict_api` decision. When that directive denies (see `restrict_api_denies`), the
/// five RESTRICTED functions render as warning + `false` instead of their normal bodies.
/// `opcache_compile_file` is deliberately NOT among them: reference PHP does not guard it —
/// VERIFIED on PHP 8.5.6, where `restrict_api=/nonexistent` still returns `true` from
/// `opcache_compile_file()` with no warning, while all five others warn and return `false`.
/// With the default empty `restrict_api` every function renders byte-identically to before.
///
/// `preload` is the compile-time `opcache.preload` block ([`preload_statistics`]), or `None` when
/// this binary does not preload (the default, and every disabled-cache binary). It only ever adds
/// the `preload_statistics` key to `opcache_get_status`; `None` renders byte-identically to before
/// the directive was supported. The UNRESOLVABLE case never reaches here — `crate::pipeline`
/// turns [`PreloadVerdict::compile_error`] into a hard compile failure BEFORE injection, exactly
/// as reference PHP fatals at startup before running a line of the script, and independently of
/// whether the program calls any OPcache function at all.
pub fn inject_if_used(
    program: Program,
    php_version: PhpVersion,
    web: bool,
    entry_path: Option<&str>,
    manifest: &[ScriptEntry],
    overrides: &[(String, String)],
    preload: Option<&PreloadStatistics>,
    strict: bool,
) -> (Program, ManifestBakeSites) {
    let mut bodies = String::new();
    let mut sites = ManifestBakeSites {
        restricted: restrict_api_denies(entry_path, php_version.version_id(), overrides),
        ..ManifestBakeSites::default()
    };

    // One compile-time decision shared by all five restricted functions.
    let restricted = sites.restricted;

    // Whether the runtime `ELEPHC_INI_*` helper block has to be emitted here. It is needed by
    // `opcache_get_configuration`'s directives array (including the RESTRICTED template's dead
    // array exit, which still has to name-resolve) and by the `opcache.*` INI dispatcher's
    // raw-string arms. Under `--web` the web prelude owns the block — emitting it here too would
    // be a redeclaration — so the flag is only ever consulted on the `!web` path below.
    let mut needs_env_helpers = detect::program_references(&program, GET_CONFIGURATION_FN)
        && !detect::program_declares(&program, GET_CONFIGURATION_FN);

    if detect::program_references(&program, GET_CONFIGURATION_FN)
        && !detect::program_declares(&program, GET_CONFIGURATION_FN)
    {
        let template = if restricted {
            splice_restrict_api_warning(RESTRICTED_GET_CONFIGURATION_TEMPLATE, true, "        ")
        } else {
            GET_CONFIGURATION_TEMPLATE.to_string()
        };
        bodies.push_str(&template.replace(
            "__OPCACHE_CONFIGURATION__",
            &render_configuration_literal(php_version, overrides),
        ));
    }

    if detect::program_references(&program, RESET_FN)
        && !detect::program_declares(&program, RESET_FN)
    {
        bodies.push_str(&if restricted {
            render_restricted_function(RESTRICTED_RESET_TEMPLATE)
        } else {
            RESET_TEMPLATE
                .replace("__OPCACHE_RESET_ENABLED__", render_reset_body(php_version, web, overrides))
        });
    }

    if detect::program_references(&program, GET_STATUS_FN)
        && !detect::program_declares(&program, GET_STATUS_FN)
    {
        // Manifest-dependent even when restricted: the restricted gate keeps the array exit
        // (as a dead branch) so the `array|false` signature survives, and that exit still
        // carries the `scripts` map and the cached-script counts.
        sites.get_status = true;
        bodies.push_str(&render_get_status_function(
            php_version,
            web,
            manifest,
            overrides,
            restricted,
            preload,
        ));
    }

    if detect::program_references(&program, IS_SCRIPT_CACHED_FN)
        && !detect::program_declares(&program, IS_SCRIPT_CACHED_FN)
    {
        // The restricted body is a bare warning + `false` with no manifest in it, so only the
        // normal body is a bake site.
        sites.is_script_cached = !restricted;
        bodies.push_str(&if restricted {
            render_restricted_function(RESTRICTED_IS_SCRIPT_CACHED_TEMPLATE)
        } else {
            render_is_script_cached_function(php_version, web, manifest, overrides)
        });
    }

    if detect::program_references(&program, INVALIDATE_FN)
        && !detect::program_declares(&program, INVALIDATE_FN)
    {
        // The restricted body warns and returns `false` with no manifest in it, so only the
        // normal body is a bake site.
        sites.invalidate = !restricted;
        bodies.push_str(&if restricted {
            render_restricted_function(RESTRICTED_INVALIDATE_TEMPLATE)
        } else {
            render_invalidate_function(php_version, web, manifest, overrides, strict)
        });
    }

    // NOT restricted in reference PHP (verified) — always the normal body.
    if detect::program_references(&program, COMPILE_FILE_FN)
        && !detect::program_declares(&program, COMPILE_FILE_FN)
    {
        sites.compile_file = true;
        bodies.push_str(&render_compile_file_function(php_version, web, manifest, overrides));
    }

    if detect::program_references(&program, IS_SCRIPT_CACHED_IN_FILE_CACHE_FN)
        && !detect::program_declares(&program, IS_SCRIPT_CACHED_IN_FILE_CACHE_FN)
    {
        // Carries no manifest either way (elephc has no file cache), so it is never a bake site.
        bodies.push_str(&if restricted {
            render_restricted_function(RESTRICTED_IS_SCRIPT_CACHED_IN_FILE_CACHE_TEMPLATE)
        } else {
            IS_SCRIPT_CACHED_IN_FILE_CACHE_TEMPLATE.to_string()
        });
    }

    // NOT restricted in reference PHP (verified) — always the normal body, and it needs no baked
    // compile-time data at all.
    if detect::program_references(&program, JIT_BLACKLIST_FN)
        && !detect::program_declares(&program, JIT_BLACKLIST_FN)
    {
        bodies.push_str(JIT_BLACKLIST_TEMPLATE);
    }

    // The in-process OPcache state block, emitted ONCE for whichever of the five state-touching
    // functions were injected. `opcache_get_status` needs it for `restart_pending` and for the
    // `scripts` map's `timestamp` / `last_used`; `opcache_reset` for the restart latch; the three
    // path functions for the discard set. Unlike the INI/env helpers this is NOT web-gated,
    // because `crate::web_prelude` never emits a copy — the OPcache functions themselves are
    // injected from here under `--web` too.
    let needs_state_helpers = [
        RESET_FN,
        GET_STATUS_FN,
        IS_SCRIPT_CACHED_FN,
        INVALIDATE_FN,
        COMPILE_FILE_FN,
    ]
    .iter()
    .any(|name| {
        detect::program_references(&program, name) && !detect::program_declares(&program, name)
    });
    if needs_state_helpers {
        bodies.push_str(&render_opcache_state_helpers());
    }

    // The `opcache.*` INI surface (`ini_get`/`ini_set`/`ini_get_all`) for CLI binaries.
    // Under `--web` the session-aware definitions in `web_prelude` own these three names
    // (and consult the shared opcache helpers themselves), so the CLI wrappers are injected
    // only when NOT web — otherwise a `--web` build would redeclare `ini_get`. Each wrapper
    // is pay-for-use with its own redeclaration guard; the shared helpers are injected once
    // whenever any of the three is.
    if !web {
        let ini_get_used = detect::program_references(&program, "ini_get")
            && !detect::program_declares(&program, "ini_get");
        let ini_set_used = detect::program_references(&program, "ini_set")
            && !detect::program_declares(&program, "ini_set");
        let ini_get_all_used = detect::program_references(&program, "ini_get_all")
            && !detect::program_declares(&program, "ini_get_all");
        if ini_get_used || ini_set_used || ini_get_all_used {
            needs_env_helpers = true;
            bodies.push_str(&render_opcache_ini_helpers(php_version, overrides));
            if ini_get_used {
                bodies.push_str(CLI_INI_GET_TEMPLATE);
            }
            if ini_set_used {
                bodies.push_str(CLI_INI_SET_TEMPLATE);
            }
            if ini_get_all_used {
                // The known-module predicate is only reachable from ini_get_all's extension
                // filter, so it is injected with it rather than with the shared helpers.
                bodies.push_str(&render_ini_module_known(false));
                bodies.push_str(CLI_INI_GET_ALL_TEMPLATE);
            }
        }
    }

    // The runtime `ELEPHC_INI_*` helper block, emitted ONCE and only on the CLI path (under
    // `--web` the web prelude bakes it — see `render_opcache_env_helpers`). It is appended last
    // because these are hoisted function declarations: order among them is irrelevant, and
    // appending keeps every earlier body byte-identical to what it rendered before.
    if !web && needs_env_helpers {
        bodies.push_str(&render_opcache_env_helpers());
    }

    if bodies.is_empty() {
        return (program, ManifestBakeSites::default());
    }

    let src = format!("<?php\n{bodies}");
    let tokens = crate::lexer::tokenize(&src).expect("opcache prelude must tokenize");
    let mut combined = crate::parser::parse_internal(&tokens).expect("opcache prelude must parse");
    combined.extend(program);
    (combined, sites)
}
