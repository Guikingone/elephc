//! Purpose:
//! Orchestrates the full PHP source to native binary compilation flow.
//! Resolves typed managed dependencies only when the selected path performs a final link.
//!
//! Called from:
//! - `crate::main()` after `crate::cli::parse_args()`.
//!
//! Key details:
//! - Pass ordering is observable: magic constants and conditionals run before resolver/name resolution and type checking.
//! - Check/EIR/assembly-only paths return before read-only native artifact resolution.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;
use std::process;
use std::time::Instant;

use crate::cli::CliConfig;
use crate::codegen::platform::Target;
use crate::codegen::Emit;
use crate::codegen::LinkRequirement;
use crate::native_deps::NativeRequirement;
use crate::span::Span;
use crate::source::SourceMode;
use crate::timings::CompileTimings;
use crate::{
    autoload, codegen, debug_info, dom_prelude, errors, exports, func_args, ir, ir_lower, ir_passes,
    lexer, linker, list_id_prelude, mysqli_prelude, name_resolver, opcache_prelude, optimize,
    parser, pdo_prelude, resolver, runtime_cache, source_map, tz_prelude, types, var_export_prelude,
    web_prelude,
};

mod backend;
mod eir_output;
mod frontend;
mod output;

use output::{dynamic_eval_capability_warning, output_paths, OutputPaths};

/// Runs the full compilation pipeline from PHP source to native binary.
/// Reads PHP source, tokenizes, parses, resolves names, type-checks, optimizes,
/// generates assembly, and links into a native binary. Exits on any error.
pub(crate) fn compile(config: CliConfig) {
    let CliConfig {
        filename,
        heap_size,
        gc_stats,
        counters,
        instrument,
        heap_debug,
        strict_opcache,
        emit_ir,
        output_dir,
        null_repr,
        emit_asm,
        emit,
        check_only,
        emit_timings,
        emit_source_map,
        emit_debug_info,
        keep_symbols,
        regalloc_linear,
        ir_opt,
        target,
        php_version,
        php_version_provenance,
        extra_link_libs,
        extra_link_paths,
        extra_frameworks,
        defines,
        strict_php,
        strict_locals,
        web,
        web_isolation,
        with_crates,
        quiet,
        ini_overrides,
    } = config;
    let filename = filename.as_str();
    crate::progress::init(quiet);
    codegen::set_null_repr(null_repr);
    // Record the PHP language profile and SAPI mode BEFORE any prelude or lowering runs: it is
    // the single source of truth for the reported version surface (`PHP_VERSION` and friends,
    // `PHP_SAPI`, `phpversion()`), which is baked far below this function's parameter list — in
    // `codegen_support::prescan::collect_constants` and in the `phpversion()` const-fold.
    codegen::set_compile_profile(php_version, web);
    crate::superglobals::set_compiling_for_web(web);
    // php's `$_SERVER` path keys name the SCRIPT, and a compiled program's script is the entry
    // it was built from. Absolute, because code that re-reads it (`autoload_runtime.php` does
    // `require $_SERVER['SCRIPT_FILENAME']`) may run from any working directory.
    crate::superglobals::set_entry_script(
        &std::fs::canonicalize(filename)
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| filename.to_string()),
    );
    crate::strict_php::set_enabled(strict_php);
    let parent = Path::new(filename).parent().unwrap_or(Path::new("."));
    let source_mode = SourceMode::from_path(Path::new(filename));
    let output_dir = output_dir.map(|raw| {
        if raw.is_absolute() {
            raw
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(&raw))
                .unwrap_or(raw)
        }
    });
    if let Some(directory) = &output_dir {
        if let Err(error) = fs::create_dir_all(directory) {
            crate::progress::clear();
            eprintln!(
                "error: cannot create output directory '{}': {}",
                directory.display(),
                error
            );
            process::exit(1);
        }
    }
    let output_paths = output_paths(filename, target, emit, output_dir.as_deref());
    // An input with no extension makes its own stem: `bin/console` compiles to `bin/console`,
    // and the link step would write the executable over the PHP file it was built from. Refuse
    // before anything is read, and name the flag that resolves it -- a destroyed source is not
    // something a later error can undo.
    for (label, produced) in [
        ("executable", &output_paths.bin),
        ("assembly", &output_paths.asm),
        ("object", &output_paths.obj),
    ] {
        if same_file(produced, Path::new(filename)) {
            eprintln!(
                "error: the {label} output would overwrite the input {}; \
                 give the build its own directory with --output-dir DIR",
                produced.display()
            );
            process::exit(1);
        }
    }
    let mut timings = CompileTimings::new(emit_timings);

    let (parsed, entry_source_unit) = frontend::read_and_parse(filename, source_mode, &defines, &mut timings);

    crate::progress::phase("autoload-build");
    let phase_started = Instant::now();
    let (autoload_registry, parsed) = autoload::Registry::build(parent, parsed);
    codegen::set_autoload_rule_count(autoload_registry.rule_count());
    for warning in autoload_registry.warnings() {
        errors::report_warning(warning);
    }
    timings.record_since("autoload-build", phase_started);

    crate::progress::phase("resolve");
    let phase_started = Instant::now();
    // `resolve_collecting_includes` also hands back the canonical path of every file the
    // resolver statically inlined — group 2 of the OPcache script manifest.
    let (ast, opcache_included_files, entry_included_sources) =
        match resolver::resolve_collecting_includes_with_defines_and_sources(
            parsed, parent, &defines,
        ) {
        Ok(resolved) => resolved,
        Err(e) => {
            crate::progress::clear();
            errors::report(&e);
            process::exit(1);
        }
    };
    let ast = autoload::collect_aliases(ast);
    timings.record_since("resolve", phase_started);


    // Report how the PHP profile is observable in THIS program, while `ast` is still the
    // user's own code: after include resolution, but before any compiler prelude is injected.
    // The `--web` prelude both calls `__elephc_php_version_id()` and defines the whole session
    // surface, so scanning any later would report every `--web` build as profile-dependent on
    // the strength of elephc's own generated code. Silent unless the profile actually changes
    // what this program computes.
    crate::php_profile::report(&ast, web, php_version, php_version_provenance);

    // Reject a profile the program's own syntax could never have run under. elephc's parser
    // accepts the whole language whatever `--php-version` says, so without this a file using
    // 8.4 property hooks compiles under `--php-version 8.2` and bakes `PHP_VERSION = "8.2.0"`
    // into a binary its source contradicts.
    if let Some(error) = crate::php_profile::floor_violation(&ast, php_version) {
        crate::progress::clear();
        errors::report(&error);
        process::exit(1);
    }

    let mut prelude_inventory = optimize::reachability::PreludeInventory::new();
    // `curl` belongs here for the same reason `pdo` does, and NOT listing it is a silent
    // no-op rather than a smaller binary: `--with-curl` force-injects the whole surface for
    // a program that reaches curl only dynamically, and declaration-reachability would then
    // delete every one of those declarations again (measured: 40 functions in, 1 out,
    // `curl_init` and `CurlHandle` both gone). See
    // `curl_prelude::reachability_tests::forcing_the_curl_group_keeps_the_whole_surface`.
    let mut forced_groups: HashSet<String> = [
        (with_crates.contains("pdo"), "pdo"),
        (with_crates.contains("mysqli"), "mysqli"),
        (with_crates.contains("tz"), "tz"),
        (with_crates.contains("image"), "image"),
        (with_crates.contains("curl"), "curl"),
        (with_crates.contains("xml"), "xml"),
    ]
    .into_iter()
    .filter_map(|(forced, group)| forced.then_some(group.to_string()))
    .collect();

    // Snapshot the USER-declared function/class names for `opcache.preload`'s
    // `preload_statistics`, taken HERE — after include resolution but BEFORE any compiler prelude
    // is injected — so the reported lists can never contain `var_export`, the PDO surface, or the
    // OPcache functions the opcache prelude itself adds. Reference PHP reports the DELTA preloading
    // added to the symbol tables, which likewise never contains a built-in. The walk visits only
    // statement lists that can host a hoisted declaration, so it is cheap on every build; it is
    // consumed only when `opcache.preload` is set (see `opcache_prelude::preload_statistics`).
    let opcache_preload_symbols = opcache_prelude::collect_preload_symbols(&ast);

    // Standard DOM classes are ordinary PHP declarations so they participate in name
    // resolution, type checking, EIR lowering, and closed-world pruning like user classes.
    let ast = dom_prelude::inject(ast);

    // Inject the PDO standard-library prelude (extern bridge + PDO classes,
    // written in elephc-PHP) only when the program references PDO, so non-PDO
    // binaries never declare the elephc_pdo externs or link the bridge.
    // Runs after include resolution so PDO usage inside includes is detected.
    // `pdo_used` is decided BEFORE injection and recorded as a PHP surface:
    // extension reporting is surface-based because the `elephc_pdo` archive
    // backs more than one PHP surface (PDO and mysqli).
    crate::progress::phase("pdo-prelude");
    let phase_started = Instant::now();
    let pdo_force = with_crates.contains("pdo");
    // Detect once, then pass the result as `force` to injection: `inject_if_used`
    // injects when `force || detect(...)`, so `force = pdo_used` reproduces the
    // exact decision without a second AST walk (the injected PDO prelude would
    // otherwise be re-scanned by the mysqli detection below too).
    let pdo_used = pdo_force || pdo_prelude::program_uses_pdo(&ast);
    let ast = if php_version == crate::web_prelude::PhpVersion::default() {
        pdo_prelude::inject_if_used(ast, pdo_used, &mut prelude_inventory)
    } else {
        pdo_prelude::inject_if_used_for_version(
            ast,
            pdo_used,
            php_version,
            &mut prelude_inventory,
        )
    };
    let mut linked_php_surfaces: Vec<String> = Vec::new();
    if pdo_used {
        linked_php_surfaces.push("PDO".to_string());
    }
    timings.record_since("pdo-prelude", phase_started);

    // Inject the mysqli prelude (a second PHP surface over the same elephc_pdo
    // bridge) only when the program references a mysqli symbol or
    // `--with-mysqli` forces it. Runs AFTER the PDO injection so the shared
    // extern block — prepended idempotently by whichever surface injects — is
    // declared exactly once, and never injects the PDO classes.
    crate::progress::phase("mysqli-prelude");
    let phase_started = Instant::now();
    let mysqli_force = with_crates.contains("mysqli");
    let mysqli_used = mysqli_force || mysqli_prelude::program_uses_mysqli(&ast);
    let ast =
        mysqli_prelude::inject_if_used(ast, mysqli_used, php_version, &mut prelude_inventory);
    if mysqli_used {
        linked_php_surfaces.push("mysqli".to_string());
    }
    timings.record_since("mysqli-prelude", phase_started);

    // Inject the timezone-introspection prelude (extern block + array marshalling,
    // written in elephc-PHP) only when the program references getLocation /
    // getTransitions / listAbbreviations or their procedural aliases, so other
    // binaries never declare the elephc_tz externs or link the bridge. Runs after
    // include resolution so usage inside includes is detected.
    crate::progress::phase("tz-prelude");
    let phase_started = Instant::now();
    let ast = tz_prelude::inject_if_used(
        ast,
        with_crates.contains("tz"),
        &mut prelude_inventory,
    );
    timings.record_since("tz-prelude", phase_started);

    // Inject the listIdentifiers-filtering prelude (a pure elephc-PHP function over
    // a baked group/country table) only when the program references
    // DateTimeZone::listIdentifiers or timezone_identifiers_list, so other binaries
    // never carry the table. Runs after include resolution so usage inside includes
    // is detected, and before name resolution, which desugars both call forms to it.
    crate::progress::phase("list-id-prelude");
    let phase_started = Instant::now();
    let ast = list_id_prelude::inject_if_used(ast, &mut prelude_inventory);
    timings.record_since("list-id-prelude", phase_started);

    // Inject the var_export prelude (a pure elephc-PHP function) only when the program
    // references var_export and does not declare its own, so other binaries carry
    // nothing. Runs after include resolution so usage inside includes is detected, and
    // before name resolution so the call resolves to the injected function.
    crate::progress::phase("var-export-prelude");
    let phase_started = Instant::now();
    let ast = var_export_prelude::inject_if_used(ast, &mut prelude_inventory);
    timings.record_since("var-export-prelude", phase_started);

    // Inject the OPcache preludes (pure elephc-PHP functions): `opcache_get_configuration()`
    // returns a compile-time array literal built from the version-keyed OPcache
    // directive matrix, and `opcache_reset()` returns the compile-time cache-enabled
    // boolean. Each is injected only when the program references it, so other binaries
    // carry nothing. Runs after include resolution so usage inside includes is detected,
    // and before name resolution so the call resolves to the injected function. The
    // reported directive set/version follows the compile target `php_version`; the
    // `opcache_reset()` result follows the SAPI (`web`): CLI disabled, web enabled.
    // Build the PLACEHOLDER OPcache script manifest: the canonicalized main entry file, every
    // statically-resolved include/require target, and eager autoload files. The PSR-4 /
    // SPL-rule class files are still unknown here — `autoload::run` produces them below, after
    // name resolution — so this manifest is completed and re-baked by
    // `opcache_prelude::bake_manifest` further down. The declarations themselves MUST be
    // injected here, before `name_resolver`, or a namespaced `opcache_get_status()` caller
    // would not resolve to them (see `opcache_prelude::bake_manifest` for the full argument).
    // The manifest feeds `opcache_get_status().scripts`, `opcache_is_script_cached`, and
    // `opcache_compile_file`.
    crate::progress::phase("opcache-prelude");
    let phase_started = Instant::now();
    let opcache_manifest = opcache_prelude::collect_manifest(
        filename,
        &opcache_included_files,
        autoload_registry.always_included_files(),
    );
    // The canonicalized entry script, resolved separately from the manifest: it is the operand
    // `opcache.restrict_api` compares its prefix against (reference PHP uses
    // `SG(request_info).path_translated`, the ENTRY script — not the executing file), and the
    // manifest deliberately drops entries it cannot stat, so its first element is not a
    // dependable stand-in. See `opcache_prelude::restrict_api_denies`.
    let opcache_entry_path = opcache_prelude::canonical_entry_path(filename);
    // `opcache.preload` is a COMPILE-TIME decision, resolved here for the same reason
    // `restrict_api` is: reference PHP preloads during STARTUP, before the script runs, and
    // elephc's INI is fixed when the binary is built. The three outcomes mirror reference exactly
    // (see `opcache_prelude::PreloadVerdict` for the verified matrix):
    // - unresolvable path with the cache enabled → HARD COMPILE ERROR, the AOT equivalent of
    //   reference's startup fatal. It fires whether or not the program calls an OPcache function,
    //   because reference's fatal does not depend on that either.
    // - resolvable but outside the compile-time script manifest → a WARNING only: preloading a file
    //   this program never includes is a legitimate configuration and must not break a build. That
    //   arm depends on the COMPLETE manifest, so it is evaluated after `autoload::run` below; only
    //   the manifest-independent compile ERROR is decided here, matching reference PHP, which
    //   fatals at startup regardless of what the script does.
    // - empty directive, or a disabled cache → nothing at all happens (reference does not preload
    //   when the accelerator is off, and does not even validate the path).
    let opcache_preload =
        opcache_prelude::preload_verdict(php_version, web, &ini_overrides, &opcache_manifest);
    if let Some(message) = opcache_preload.compile_error() {
        errors::report(&errors::CompileError::new(Span::new(0, 0), &message).with_file(filename.to_string()));
        process::exit(1);
    }
    let opcache_preload_statistics = opcache_prelude::preload_statistics(
        &opcache_preload,
        &opcache_manifest,
        &opcache_preload_symbols,
    );
    let (ast, opcache_bake_sites) = opcache_prelude::inject_if_used(
        ast,
        php_version,
        web,
        opcache_entry_path.as_deref(),
        &opcache_manifest,
        &ini_overrides,
        opcache_preload_statistics.as_ref(),
        strict_opcache,
        &mut prelude_inventory,
    );
    timings.record_since("opcache-prelude", phase_started);

    // Inject the image standard-library prelude (elephc_image externs + GD/Exif/
    // Imagick/Gmagick/Cairo surface, written in elephc-PHP) only when the program
    // references an image symbol, so non-image binaries never declare the
    // elephc_image externs or link the bridge. Runs after include resolution so
    // image usage inside includes is detected.
    crate::progress::phase("image-prelude");
    let phase_started = Instant::now();
    let ast = crate::image_prelude::inject_if_used(
        ast,
        with_crates.contains("image"),
        &mut prelude_inventory,
    );
    timings.record_since("image-prelude", phase_started);

    // Inject the incremental-hashing prelude (the `HashContext` class and the
    // `hash_init`/`hash_update`/`hash_final`/`hash_copy` wrappers over the internal
    // `__elephc_hash_ctx_*` builtins) only when the program references that surface,
    // so non-hashing binaries never declare `HashContext` and never link
    // `-lelephc_crypto`. Runs after include resolution so hashing inside includes is
    // detected, and before name resolution so a namespaced caller resolves to it.
    crate::progress::phase("hash-prelude");
    let phase_started = Instant::now();
    let ast = crate::hash_prelude::inject_if_used(ast, false, &mut prelude_inventory);
    timings.record_since("hash-prelude", phase_started);

    // Inject the `ext/curl` prelude (the `CurlHandle` class and the `curl_*` wrappers over
    // the internal `__elephc_curl_*` builtins) only when the program references that
    // surface, so non-curl binaries never declare `CurlHandle`, never link
    // `-lelephc_curl`, and never require the managed native `curl` package. Runs after
    // the hash prelude (both are order-independent declaration-only preludes) and before
    // name resolution so a namespaced caller resolves to it. `--with-curl` forces the
    // injection for a program that only reaches curl dynamically. The version selects the
    // curl SURFACE too: `curl_multi_get_handles()` is 8.5-only.
    crate::progress::phase("curl-prelude");
    let phase_started = Instant::now();
    let ast = if php_version == crate::php_version::PhpVersion::default() {
        crate::curl_prelude::inject_if_used(
            ast,
            with_crates.contains("curl"),
            &mut prelude_inventory,
        )
    } else {
        crate::curl_prelude::inject_if_used_for_version(
            ast,
            with_crates.contains("curl"),
            php_version,
            &mut prelude_inventory,
        )
    };
    timings.record_since("curl-prelude", phase_started);

    // Inject the `ext/xml` / `ext/xmlwriter` prelude (the `XMLParser` and `XMLWriter`
    // classes and the `xml_*` / `xmlwriter_*` wrappers over the `elephc_xml` extern block)
    // only when the program references that surface, so XML-free binaries never declare
    // the classes and never link `-lelephc_xml`. Order-independent like the hash and curl
    // preludes; `--with-xml` forces the injection for opaque dynamic use.
    crate::progress::phase("xml-prelude");
    let phase_started = Instant::now();
    let ast = crate::xml_prelude::inject_if_used(
        ast,
        with_crates.contains("xml"),
        &mut prelude_inventory,
    );
    timings.record_since("xml-prelude", phase_started);

    crate::progress::phase("web-prelude");
    let phase_started = Instant::now();
    let ast = web_prelude::inject_if_web(
        ast,
        web,
        php_version,
        &ini_overrides,
        Path::new(filename),
        &mut prelude_inventory,
    );
    timings.record_since("web-prelude", phase_started);

    // Inject the PHP version-surface functions (`zend_version`, `php_sapi_name`,
    // `ini_restore`) the program actually references. Runs AFTER the web prelude so a
    // `--web` build's own declarations are already present and the redeclaration guard sees
    // them, and before name resolution so a namespaced caller resolves to the injection.
    crate::progress::phase("version-prelude");
    let phase_started = Instant::now();
    let ast = crate::version_prelude::inject_if_used(
        ast,
        php_version,
        &mut prelude_inventory,
    );
    timings.record_since("version-prelude", phase_started);

    crate::progress::phase("name-resolve");
    let phase_started = Instant::now();
    let ast = match name_resolver::resolve(ast) {
        Ok(resolved) => resolved,
        Err(e) => {
            crate::progress::clear();
            errors::report(&e);
            process::exit(1);
        }
    };
    // The entry file's own interfaces are the last declarations with no activation event: neither
    // the resolver's include stripping nor the autoload pass has seen them.
    let ast =
        autoload::activate_entry_interface_declarations(ast, &entry_source_unit.canonical_path);
    timings.record_since("name-resolve", phase_started);

    // php preloads at STARTUP, before any request, and what survives is the symbol table it built.
    // A compiled binary has no startup, so the AOT equivalent is to take the preloaded files'
    // DECLARATIONS into the closed world and to run none of the preload graph's own statements.
    // They go in ahead of the entry's, and any name the entry declares for itself is dropped from
    // them first: both halves reach `vendor/autoload.php`, and php declares each class once.
    // Placed after NAME RESOLUTION, because that is what makes the two halves comparable: before
    // it the entry still spells a declaration `ClassLoader` while the preload half, resolved on
    // its own pass, already spells it `Composer\\Autoload\\ClassLoader`.
    crate::progress::phase("opcache-preload");
    let phase_started = Instant::now();
    let ast = match preloaded_declarations(&ini_overrides, parent, &defines) {
        Ok(Some(preloaded)) => {
            let declared = crate::opcache_preload_sources::declared_names(&ast);
            let mut combined =
                crate::opcache_preload_sources::without_redeclarations(preloaded, &declared);
            combined.extend(ast);
            combined
        }
        Ok(None) => ast,
        Err(e) => {
            crate::progress::clear();
            errors::report(&e);
            process::exit(1);
        }
    };
    timings.record_since("opcache-preload", phase_started);

    crate::progress::phase("autoload-run");
    let phase_started = Instant::now();
    // `run_collecting_included` also hands back the canonical path of every file the autoload
    // pass loaded — eager files, PSR-4 / SPL-rule class files, and their own
    // include targets: group 3 of the OPcache script manifest, and the last one to become
    // knowable.
    let (ast, opcache_autoloaded_files, declaration_source_files) =
        match autoload::run_collecting_included_with_defines_and_sources(
            ast,
            parent,
            &autoload_registry,
            &defines,
        ) {
            Ok(resolved) => resolved,
            Err(e) => {
                crate::progress::clear();
                errors::report(&e);
                process::exit(1);
            }
        };
    timings.record_since("autoload-run", phase_started);

    // Inject compatibility functions only after autoload expansion has exposed the complete
    // closed-world program. Their EIR routing is type-directed, so the declarations must be
    // present before checking even when the triggering call came from an autoloaded class.
    crate::progress::phase("compat-preludes");
    let phase_started = Instant::now();
    // PHP's error/exception-handling surface exists in EVERY SAPI, and without this a build
    // off `--web` has none of it: `error_reporting()` is an undefined function, which alone
    // stops a console entry point. The declarations are the ones the `--web` prelude injects,
    // so this is a no-op under `--web`; off it, the injection is pay-for-use on what the
    // program names.
    //
    // It belongs HERE and not in the web-prelude phase, for the reason this phase exists: the
    // program that names the surface is usually not the entry file. Symfony's console calls
    // `error_reporting()` from `Runtime\Internal\BasicErrorHandler`, a class the autoload pass
    // splices in at `autoload-run` — which is AFTER the web-prelude phase, so the gate saw a
    // 21-line entry that mentions nothing and injected nothing, and the binary died at run
    // time on the undefined function. A `require`d file was visible and an autoloaded class
    // was not, which is exactly the distinction this phase's existing comment draws.
    let ast = crate::error_handling_prelude::inject_if_used(
        ast,
        web,
        Path::new(filename),
        &mut prelude_inventory,
    );
    // The engine-diagnostic dispatch pair is reached from the generated runtime, never from
    // PHP, so declaration reachability would delete it. Forcing its group is what keeps
    // `set_error_handler()` able to see a warning raised by compiled code.
    if prelude_inventory
        .groups
        .contains_key(web_prelude::DIAG_DISPATCH_GROUP)
    {
        forced_groups.insert(web_prelude::DIAG_DISPATCH_GROUP.to_string());
    }
    // `__elephc_shutdown_run` is likewise reached from EMITTED CODE and never from PHP off
    // `--web`: `lower_exit` and `emit_main_epilogue` call it by symbol. Forcing its group is
    // what keeps `register_shutdown_function()`'s queue drained at `exit()` and at the normal
    // end of the script. The group only exists when the program spelled the registration
    // function, so this stays pay-for-use.
    if prelude_inventory
        .groups
        .contains_key(crate::error_handling_prelude::SHUTDOWN_RUN_GROUP)
    {
        forced_groups.insert(crate::error_handling_prelude::SHUTDOWN_RUN_GROUP.to_string());
    }
    // PHP's `ini_get()` / `ini_set()` / `ini_get_all()` exist in every SAPI. Off `--web` elephc
    // has had them all along (the `opcache.*`-backed wrappers in `opcache_prelude`), but their
    // only gate ran in the `opcache-prelude` phase — before autoload expansion — so a program
    // that names them from an AUTOLOADED class got nothing and died at run time with `Call to
    // undefined function ini_set()`. Measured on `examples/symfony-app/bin/console`. This second
    // gate reads the COMPLETE program and injects only what the first one could not see; it is
    // a no-op under `--web` and a no-op whenever the early site already fired.
    let ast = opcache_prelude::inject_cli_ini_if_used(
        ast,
        php_version,
        &ini_overrides,
        web,
        &mut prelude_inventory,
    );
    let ast = crate::assert_prelude::inject_if_used(ast);
    let ast = crate::array_merge_prelude::inject_if_used(ast, &mut prelude_inventory);
    if prelude_inventory.groups.contains_key("array_merge") {
        forced_groups.insert("array_merge".to_string());
    }
    let ast = crate::array_reduce_prelude::inject_if_used(ast);
    let ast = crate::filter_var_prelude::inject_if_used(ast, &mut prelude_inventory);
    if prelude_inventory
        .groups
        .contains_key(crate::filter_var_prelude::FILTER_VAR_GROUP)
    {
        forced_groups.insert(crate::filter_var_prelude::FILTER_VAR_GROUP.to_string());
    }
    let ast = crate::backend_gap_prelude::inject_if_used(ast, &mut prelude_inventory);
    if prelude_inventory
        .groups
        .contains_key(crate::backend_gap_prelude::BACKEND_GAP_GROUP)
    {
        forced_groups.insert(crate::backend_gap_prelude::BACKEND_GAP_GROUP.to_string());
    }
    timings.record_since("compat-preludes", phase_started);

    // Desugar PHP's argument-introspection constructs (`func_num_args`, `func_get_args`,
    // `func_get_arg`) into plain PHP: every function scope that uses one gains the hidden
    // `mixed ...$__elephc_func_args` parameter, so the surplus positional arguments PHP
    // allows are collected by the existing variadic machinery. Runs after `autoload::run`
    // so autoloaded declarations are covered too — which means call names are already
    // resolved here and are matched on their unqualified last segment — and before the AST
    // optimizer and the checker, which then only ever see ordinary PHP.
    crate::progress::phase("func-args");
    let phase_started = Instant::now();
    let ast = match func_args::desugar(ast) {
        Ok(desugared) => desugared,
        Err(e) => {
            crate::progress::clear();
            errors::report(&e);
            process::exit(1);
        }
    };
    timings.record_since("func-args", phase_started);

    // Complete the OPcache script manifest now that all three groups exist, and re-render the
    // manifest-dependent functions injected above against it. This is a pure substitution of
    // already-declared, already-name-resolved top-level functions, so it cannot disturb the
    // name resolution that has already happened (see `opcache_prelude::bake_manifest`). It runs
    // before `optimize::fold_constants` so the baked literals meet every later pass exactly as
    // the placeholder ones would have.
    crate::progress::phase("opcache-manifest-bake");
    let phase_started = Instant::now();
    let opcache_manifest = opcache_prelude::collect_manifest(
        filename,
        &opcache_included_files,
        &opcache_autoloaded_files,
    );
    // Re-decide `opcache.preload` against the complete manifest. Only the `in_manifest` arm can
    // differ from the verdict taken above (the directive, the SAPI gate and the path resolution
    // are all manifest-independent), so this second call exists purely to emit the
    // outside-the-manifest WARNING against the truthful set — reporting it against the
    // placeholder manifest would warn about files that are, in fact, compiled in.
    let opcache_preload =
        opcache_prelude::preload_verdict(php_version, web, &ini_overrides, &opcache_manifest);
    if let Some(message) = opcache_preload.compile_warning() {
        errors::report_warning(&errors::CompileWarning::new(Span::new(0, 0), &message));
    }
    let opcache_preload_statistics = opcache_prelude::preload_statistics(
        &opcache_preload,
        &opcache_manifest,
        &opcache_preload_symbols,
    );
    let ast = opcache_prelude::bake_manifest(
        ast,
        &opcache_bake_sites,
        php_version,
        web,
        &opcache_manifest,
        &ini_overrides,
        opcache_preload_statistics.as_ref(),
        strict_opcache,
    );
    timings.record_since("opcache-manifest-bake", phase_started);

    crate::progress::phase("opt-fold");
    let phase_started = Instant::now();
    let ast = optimize::fold_constants_for_target(ast, target);
    timings.record_since("opt-fold", phase_started);

    crate::progress::phase("typecheck");
    let phase_started = Instant::now();
    // The autoload registry has already CONSUMED (and removed from the AST) every
    // `spl_autoload_register` call it could collect, so the checker cannot rediscover them by
    // scanning. It needs to know, because a registered loader means an unknown class name is a
    // name that arrives at run time rather than one that does not exist — see
    // `Checker::program_defers_unknown_classes`.
    let check_options = types::CheckOptions {
        strict_locals,
        registers_autoloader: autoload_registry.rule_count() > 0,
    };
    let mut check_result = match types::check_with_target_and_options(&ast, target, check_options) {
        Ok(result) => result,
        Err(e) => {
            crate::progress::clear();
            // Name the file each error was written in. The autoload pass spliced every discovered
            // file into one program, so the `line:col` the checker reports lives in a coordinate
            // space shared by hundreds of files and identifies nothing on its own. The checker
            // recorded the enclosing declaration; `declaration_source_files` is the map from that
            // name to its path — the same one `Reflection*::getFileName()` reads.
            let declaration_paths = declaration_source_file_paths(
                &declaration_source_files,
                &entry_included_sources,
            );
            errors::report(&e.resolve_declaration_files(&declaration_paths));
            process::exit(1);
        }
    };
    timings.record_since("typecheck", phase_started);
    for warning in &check_result.warnings {
        errors::report_warning(warning);
    }
    if !target.supports_current_backend() {
        crate::progress::clear();
        eprintln!(
            "Target '{}' is recognized, but it is outside the current supported target matrix",
            target
        );
        process::exit(1);
    }

    crate::progress::phase("exports-scan");
    let phase_started = Instant::now();
    let exported_functions = match exports::collect(&ast, &check_result.functions) {
        Ok(exports) => exports,
        Err(e) => {
            crate::progress::clear();
            errors::report(&e.with_file(filename.to_string()));
            process::exit(1);
        }
    };
    timings.record_since("exports-scan", phase_started);
    if matches!(emit, Emit::Executable)
        && !check_only
        && !emit_ir
        && !exported_functions.is_empty()
    {
        let names: Vec<&str> = exported_functions.keys().map(String::as_str).collect();
        eprintln!(
            "warning: ignoring #[Export] on functions {:?} — --emit cdylib is required to expose them",
            names
        );
    }

    if check_only && exported_functions.is_empty() {
        crate::progress::clear();
        timings.report();
        crate::progress::finish_ok(&format!("Checked '{}'", filename), timings.elapsed());
        return;
    }

    crate::progress::phase("opt-prop");
    let phase_started = Instant::now();
    let post_typecheck_optimizer = optimize::PostTypecheckOptimizer::new_with_type_metadata(
        &ast,
        &check_result.functions,
        &check_result.classes,
        &check_result.interfaces,
    );
    // Substituting a literal for a read of a local the checker boxed as `mixed` would hand EIR
    // lowering a concrete type the checker never approved for that name, so the pass is told which
    // names those are and refuses to record a fact for them.
    let ast = post_typecheck_optimizer.propagate(ast, check_result.mixed_storage_local_names());
    timings.record_since("opt-prop", phase_started);

    crate::progress::phase("opt-post");
    let phase_started = Instant::now();
    // Pruning and normalization both run the single-case switch rewrite, which materializes the
    // default body into BOTH branches of the synthesized `if` with the original's spans. The
    // checker's local-binding decisions are keyed BY SPAN, so these phases are told which spans
    // carry one and the rewrite vetoes itself rather than duplicating a decision.
    let ast = post_typecheck_optimizer.prune(ast, check_result.local_binding_decision_spans());
    timings.record_since("opt-post", phase_started);

    crate::progress::phase("opt-norm");
    let phase_started = Instant::now();
    let ast = post_typecheck_optimizer.normalize(ast, check_result.local_binding_decision_spans());
    timings.record_since("opt-norm", phase_started);

    crate::progress::phase("dce");
    let phase_started = Instant::now();
    // Tail-sinking clones the tail of an `if`/`switch`/`try` into every branch, and a clone keeps
    // the original's spans — the same span-keyed hazard, in the other pass that clones.
    let ast = post_typecheck_optimizer
        .eliminate_dead_code(ast, check_result.local_binding_decision_spans());
    timings.record_since("dce", phase_started);

    crate::progress::phase("decl-reach");
    let phase_started = Instant::now();
    let metadata_trace_target = std::env::var("ELEPHC_METADATA_TRACE").ok();
    let metadata_trace_before = metadata_trace_target.as_deref().map(|target| {
        let normalized = target.trim_start_matches('\\');
        (
            check_result
                .interfaces
                .keys()
                .any(|name| name.eq_ignore_ascii_case(normalized)),
            check_result
                .classes
                .keys()
                .any(|name| name.eq_ignore_ascii_case(normalized)),
        )
    });
    let exported_function_names: HashSet<String> = exported_functions.keys().cloned().collect();
    let ast = optimize::prune_unreachable_declarations(
        ast,
        &mut check_result,
        optimize::reachability::PruneOptions {
            inventory: &prelude_inventory,
            forced_groups: &forced_groups,
            exported_functions: &exported_function_names,
            eval_forced: with_crates.contains("eval"),
        },
    );
    timings.record_since("decl-reach", phase_started);
    if let (Some(target), Some((interface_before, class_before))) =
        (metadata_trace_target.as_deref(), metadata_trace_before)
    {
        let normalized = target.trim_start_matches('\\');
        let interface_after = check_result
            .interfaces
            .keys()
            .any(|name| name.eq_ignore_ascii_case(normalized));
        let class_after = check_result
            .classes
            .keys()
            .any(|name| name.eq_ignore_ascii_case(normalized));
        eprintln!(
            "[elephc-metadata-trace] target={normalized} interface_before={interface_before} interface_after={interface_after} class_before={class_before} class_after={class_after}"
        );
    }
    codegen::prepare_declared_name_order(
        &ast,
        &check_result.classes,
        &check_result.interfaces,
    );

    crate::progress::phase("ir-lower");
    let phase_started = Instant::now();
    let mut source_units = BTreeMap::new();
    for unit in std::iter::once(entry_source_unit)
        .chain(entry_included_sources.source_units.into_values())
        .chain(declaration_source_files.source_units.into_values())
    {
        if let Err(error) = unit.insert_into(&mut source_units, Span::dummy()) {
            crate::progress::clear();
            errors::report(&error);
            process::exit(1);
        }
    }
    // `run_collecting_included` reports every source it actually loaded. Its
    // declaration map can legitimately omit a file whose declarations were
    // later eliminated, but a typed activation event still needs that physical
    // source identity. Fill only absent manifest paths; duplicate snapshots keep
    // the existing conflict check above rather than silently replacing content.
    let activation_source_paths = classlike_activation_source_paths(&ast);
    for path in opcache_included_files
        .iter()
        .chain(opcache_autoloaded_files.iter())
        .chain(activation_source_paths.iter())
    {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
        if source_units.contains_key(&canonical) {
            continue;
        }
        let source = match crate::source::read_physical_source(path) {
            Ok(source) => source,
            Err(error) => {
                crate::progress::clear();
                errors::report(&crate::errors::CompileError::new(
                    Span::dummy(),
                    &format!("Autoload source snapshot: cannot read '{}': {}", path.display(), error),
                ));
                process::exit(1);
            }
        };
        let unit = resolver::SourceUnit {
            canonical_path: canonical,
            mode: SourceMode::from_path(path),
            source: source.into(),
        };
        if let Err(error) = unit.insert_into(&mut source_units, Span::dummy()) {
            crate::progress::clear();
            errors::report(&error);
            process::exit(1);
        }
    }
    let source_catalog = match ir::SourceCatalog::from_units(source_units.into_values()) {
        Ok(catalog) => catalog,
        Err(error) => {
            crate::progress::clear();
            errors::report(&error);
            process::exit(1);
        }
    };
    let mut ir_module = match ir_lower::lower_program_with_source_catalog(
        &ast,
        &check_result,
        target,
        Path::new(filename),
        web,
        source_catalog,
    ) {
        Ok(module) => module,
        Err(err) => {
            crate::progress::clear();
            eprintln!("EIR lowering error: {}", err);
            process::exit(1);
        }
    };
    ir_module.declared_class_source_files = declaration_source_files.class_likes;
    ir_module.declared_function_source_files = declaration_source_files.functions;
    // The autoload pass PERFORMED these inclusions: it opened each file and spliced its
    // declarations into the program. At runtime they are already-included files, and an
    // `include_once` reaching one through a computed path must answer "already included"
    // instead of re-running it into a redeclaration fatal.
    crate::autoload::record_compile_time_inclusions(
        &mut ir_module,
        &ast,
        &opcache_autoloaded_files,
    );
    ir_module.required_runtime_features.class_introspection |= ir_module
        .interface_infos
        .values()
        .any(|info| info.declaration_span != crate::span::Span::dummy());
    timings.record_since("ir-lower", phase_started);

    // EIR owns every backend-visible declaration, signature and source-identity datum now.
    // Keeping the optimized AST and the complete checker graph through assembly doubles the
    // peak working set on large applications, even though the backend only still needs the
    // requested native library names. Move that small list out, then release both graphs before
    // EIR optimization/codegen starts.
    let required_libraries = std::mem::take(&mut check_result.required_libraries);
    drop(ast);
    drop(check_result);

    if emit_ir {
        eir_output::emit(ir_module, filename, ir_opt, &exported_functions, &mut timings);
        return;
    }

    if emit.is_library() || (check_only && !exported_functions.is_empty()) {
        if let Err(error) = exports::validate_cdylib_call_graph(&ir_module, &exported_functions) {
            crate::progress::clear();
            errors::report(&error.with_file(filename.to_string()));
            process::exit(1);
        }
    }

    if check_only {
        crate::progress::clear();
        timings.report();
        crate::progress::finish_ok(&format!("Checked '{}'", filename), timings.elapsed());
        return;
    }

    crate::progress::phase("ir-opt");
    let phase_started = Instant::now();
    if ir_opt {
        ir_passes::optimize_module(&mut ir_module);
    }
    timings.record_since("ir-opt", phase_started);

    backend::emit_and_link(backend::BackendInputs {
        filename,
        with_crates: &with_crates,
        linked_php_surfaces: &linked_php_surfaces,
        ir_module,
        web,
        web_isolation,
        extra_link_libs: &extra_link_libs,
        extra_link_paths: &extra_link_paths,
        extra_frameworks: &extra_frameworks,
        required_libraries: &required_libraries,
        target,
        emit,
        heap_size,
        gc_stats,
        counters,
        instrument,
        heap_debug,
        exported_functions: &exported_functions,
        regalloc_linear,
        emit_debug_info,
        keep_symbols,
        output_paths: &output_paths,
        emit_source_map,
        emit_asm,
        timings: &mut timings,
    });
}


/// Returns whether two paths name the same file on disk, or the same text when either is absent.
///
/// `canonicalize` resolves `.`, `..` and symlinks, which a textual comparison misses --
/// `./bin/console` and `bin/console` are the same file. It fails on a path that does not exist
/// yet, which is the normal case for an output, so the textual comparison stays as the fallback.
fn same_file(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

/// Returns every physical source path named by a retained typed source marker.
///
/// The resolver has already established these paths while it still owned the
/// original source. This final inventory is intentionally after optimization:
/// only events that can reach EIR need a SourceId, and it closes the gap where
/// an autoload manifest does not retain a source unit for a nested declaration
/// or include-once marker.
fn classlike_activation_source_paths(
    program: &crate::parser::ast::Program,
) -> std::collections::BTreeSet<std::path::PathBuf> {
    use crate::parser::ast::StmtKind;

    fn collect(
        statements: &[crate::parser::ast::Stmt],
        paths: &mut std::collections::BTreeSet<std::path::PathBuf>,
    ) {
        for statement in statements {
            match &statement.kind {
                StmtKind::ClassLikeActivate { source_path, .. } => {
                    paths.insert(source_path.clone());
                }
                StmtKind::IncludeOnceMark { source_path } => {
                    paths.insert(source_path.clone());
                }
                StmtKind::NamespaceBlock { body, .. }
                | StmtKind::Synthetic(body)
                | StmtKind::While { body, .. }
                | StmtKind::DoWhile { body, .. }
                | StmtKind::Foreach { body, .. } => collect(body, paths),
                StmtKind::IncludeOnceGuard { source_path, body } => {
                    paths.insert(source_path.clone());
                    collect(body, paths);
                }
                StmtKind::If {
                    then_body,
                    elseif_clauses,
                    else_body,
                    ..
                } => {
                    collect(then_body, paths);
                    for (_, body) in elseif_clauses {
                        collect(body, paths);
                    }
                    if let Some(body) = else_body {
                        collect(body, paths);
                    }
                }
                StmtKind::IfDef {
                    then_body,
                    else_body,
                    ..
                } => {
                    collect(then_body, paths);
                    if let Some(body) = else_body {
                        collect(body, paths);
                    }
                }
                StmtKind::For {
                    init,
                    update,
                    body,
                    ..
                } => {
                    if let Some(init) = init {
                        collect(std::slice::from_ref(init.as_ref()), paths);
                    }
                    if let Some(update) = update {
                        collect(std::slice::from_ref(update.as_ref()), paths);
                    }
                    collect(body, paths);
                }
                StmtKind::FunctionDecl { body, .. } => collect(body, paths),
                StmtKind::ClassDecl { methods, .. }
                | StmtKind::InterfaceDecl { methods, .. }
                | StmtKind::TraitDecl { methods, .. }
                | StmtKind::EnumDecl { methods, .. } => {
                    for method in methods {
                        collect(&method.body, paths);
                    }
                }
                StmtKind::Switch { cases, default, .. } => {
                    for (_, body) in cases {
                        collect(body, paths);
                    }
                    if let Some(body) = default {
                        collect(body, paths);
                    }
                }
                StmtKind::Try {
                    try_body,
                    catches,
                    finally_body,
                } => {
                    collect(try_body, paths);
                    for catch in catches {
                        collect(&catch.body, paths);
                    }
                    if let Some(body) = finally_body {
                        collect(body, paths);
                    }
                }
                _ => {}
            }
        }
    }

    let mut paths = std::collections::BTreeSet::new();
    collect(program, &mut paths);
    paths
}

/// Returns the declarations `opcache.preload` supplies, or `None` when the directive is unset.
///
/// The directive is the only input: a program that does not set it compiles exactly as before, and
/// one that does gets the same world a preloaded php-fpm serves. Nothing here knows what framework
/// wrote the file.
fn preloaded_declarations(
    ini_overrides: &[(String, String)],
    base_dir: &Path,
    defines: &std::collections::HashSet<String>,
) -> Result<Option<crate::parser::ast::Program>, errors::CompileError> {
    let Some((_, directive)) = ini_overrides
        .iter()
        .rev()
        .find(|(key, _)| key.eq_ignore_ascii_case("opcache.preload"))
    else {
        return Ok(None);
    };
    let Some(path) = crate::opcache_preload_sources::resolve_preload_path(directive, base_dir)
    else {
        return Ok(None);
    };
    if !path.exists() {
        // php treats an unreadable preload file as a startup FATAL, and the AOT equivalent of
        // "not there" is a compile error -- the same verdict `opcache_prelude::preload_verdict`
        // already reaches for the directive's own reporting.
        return Err(errors::CompileError::new(
            crate::span::Span::dummy(),
            &format!("opcache.preload: '{}' does not exist", path.display()),
        ));
    }
    crate::opcache_preload_sources::preload_declarations(&path, base_dir, defines).map(Some)
}

/// Merges the autoloaded and entry-file declaration maps into one name-to-path lookup.
///
/// Both halves are needed: the entry file's own declarations never pass through the autoload
/// registry, and the autoloaded ones are the bulk of a real application. A name declared in both
/// resolves to the autoloaded path, which is the one the program actually runs.
fn declaration_source_file_paths(
    autoloaded: &autoload::DeclarationSourceFiles,
    entry: &crate::resolver::IncludedDeclarationSources,
) -> std::collections::HashMap<String, String> {
    let mut paths = std::collections::HashMap::new();
    let halves = [
        (&entry.class_likes, &entry.functions),
        (&autoloaded.class_likes, &autoloaded.functions),
    ];
    for (class_likes, functions) in halves {
        paths.extend(
            class_likes
                .iter()
                .chain(functions.iter())
                .map(|(name, path)| (name.clone(), path.clone())),
        );
    }
    paths
}
