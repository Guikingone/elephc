//! Purpose:
//! Orchestrates the full PHP source to native binary compilation flow.
//! Runs frontend passes, semantic checks, optimizations, runtime preparation, codegen, and linking in order.
//!
//! Called from:
//! - `crate::main()` after `crate::cli::parse_args()`.
//!
//! Key details:
//! - Pass ordering is observable: magic constants and conditionals run before resolver/name resolution and type checking.

use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Instant;

use crate::cli::CliConfig;
use crate::codegen::platform::{Platform, Target};
use crate::codegen::Emit;
use crate::timings::CompileTimings;
use crate::{
    autoload, codegen, conditional, debug_info, errors, exports, ir, ir_lower, ir_passes, lexer,
    linker, list_id_prelude, magic_constants, name_resolver, optimize, parser, pdo_prelude,
    resolver, runtime_cache, shutdown_prelude, source_map, tree_shake, tz_prelude, types,
    var_export_prelude, web_prelude,
};

/// Holds the paths for all compilation output files (assembly, object, binary, source map).
struct OutputPaths {
    asm: PathBuf,
    obj: PathBuf,
    bin: PathBuf,
    source_map: PathBuf,
}

/// Runs the full compilation pipeline from PHP source to native binary.
/// Reads PHP source, tokenizes, parses, resolves names, type-checks, optimizes,
/// generates assembly, and links into a native binary. Exits on any error.
pub(crate) fn compile(config: CliConfig) {
    let CliConfig {
        filename,
        heap_size,
        gc_stats,
        heap_debug,
        emit_ir,
        null_repr,
        emit_asm,
        emit,
        check_only,
        emit_timings,
        emit_source_map,
        emit_debug_info,
        regalloc_linear,
        ir_opt,
        tree_shake,
        target,
        php_version,
        mut extra_link_libs,
        extra_link_paths,
        extra_frameworks,
        defines,
        strict_php,
        web,
        with_crates,
    } = config;
    let filename = filename.as_str();
    codegen::set_null_repr(null_repr);
    crate::strict_php::set_enabled(strict_php);
    let parent = Path::new(filename).parent().unwrap_or(Path::new("."));
    // Autoload (Composer PSR-4/classmap + `vendor/`) is resolved relative to the
    // composer project root, which may sit ABOVE the entry directory (e.g. Symfony's
    // `public/index.php` with composer.json at the app root). Walk up to find it.
    // Include resolution still uses `parent` (the entry dir) so relative includes in
    // the entry file resolve against its own directory, unaffected by this.
    let autoload_root = autoload::find_composer_project_root(parent);
    let output_paths = output_paths(filename, target, emit);
    let mut timings = CompileTimings::new(emit_timings);

    let phase_started = Instant::now();
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", filename, e);
            process::exit(1);
        }
    };
    timings.record_since("read", phase_started);

    let phase_started = Instant::now();
    let tokens = match lexer::tokenize(&source) {
        Ok(tokens) => tokens,
        Err(e) => {
            errors::report(&e.with_file(filename.to_string()));
            process::exit(1);
        }
    };
    timings.record_since("tokenize", phase_started);

    let phase_started = Instant::now();
    let parsed = match parser::parse(&tokens) {
        Ok(ast) => ast,
        Err(e) => {
            errors::report(&e.with_file(filename.to_string()));
            process::exit(1);
        }
    };
    timings.record_since("parse", phase_started);

    let phase_started = Instant::now();
    let main_file_path = Path::new(filename).to_path_buf();
    let parsed = magic_constants::substitute_file_and_scope_constants(parsed, &main_file_path);
    timings.record_since("magic-constants", phase_started);

    // Snapshot which top-level classes/enums/functions are declared directly IN THIS FILE,
    // before `include`/`require` merging (resolver) or autoloaded-library splicing (autoload)
    // can add declarations from OTHER files into `parsed`. Backs `ReflectionClass::getFileName()`
    // / `ReflectionFunction::getFileName()` (see `scan_reflection_source_files`): declarations
    // that only exist in an included/autoloaded file are simply absent from these maps, so
    // `getFileName()` reports PHP's `false` for them rather than guessing. Canonicalized (symlinks
    // resolved) to match PHP's own `getFileName()`/`__FILE__` behavior (see
    // `crate::magic_constants::file_pass::substitute_file_constants`, which canonicalizes the
    // same way).
    let canonical_main_file_path = main_file_path
        .canonicalize()
        .unwrap_or_else(|_| main_file_path.clone());
    let (class_source_files, function_source_files) =
        resolver::scan_reflection_source_files(&parsed, &canonical_main_file_path);
    // Strict-PHP audit of the main file: after magic-constant substitution
    // (matching the include/autoload audit sites) and before
    // `conditional::apply` consumes `ifdef` nodes, so every elephc-only
    // construct is reported with its span. Included and autoloaded user files
    // are audited where they are parsed (resolver / autoloader), so injected
    // compiler preludes are never audited.
    if let Err(e) = crate::strict_php::check_file(&parsed, filename) {
        errors::report(&e);
        process::exit(1);
    }

    let parsed = conditional::apply(parsed, &defines);

    let phase_started = Instant::now();
    let (autoload_registry, parsed) = autoload::Registry::build(&autoload_root, parsed);
    codegen::set_autoload_rule_count(autoload_registry.rule_count());
    for warning in autoload_registry.warnings() {
        errors::report_warning(warning);
    }
    timings.record_since("autoload-build", phase_started);

    let phase_started = Instant::now();
    let ast = match resolver::resolve(parsed, parent) {
        Ok(resolved) => resolved,
        Err(e) => {
            errors::report(&e);
            process::exit(1);
        }
    };
    let ast = autoload::collect_aliases(ast);
    timings.record_since("resolve", phase_started);

    // Inject the PDO standard-library prelude (extern bridge + PDO classes,
    // written in elephc-PHP) only when the program references PDO, so non-PDO
    // binaries never declare the elephc_pdo externs or link the bridge.
    // Runs after include resolution so PDO usage inside includes is detected.
    let phase_started = Instant::now();
    let ast = pdo_prelude::inject_if_used(ast, with_crates.contains("pdo"));
    timings.record_since("pdo-prelude", phase_started);

    // Inject the timezone-introspection prelude (extern block + array marshalling,
    // written in elephc-PHP) only when the program references getLocation /
    // getTransitions / listAbbreviations or their procedural aliases, so other
    // binaries never declare the elephc_tz externs or link the bridge. Runs after
    // include resolution so usage inside includes is detected.
    let phase_started = Instant::now();
    let ast = tz_prelude::inject_if_used(ast, with_crates.contains("tz"));
    timings.record_since("tz-prelude", phase_started);

    // Inject the listIdentifiers-filtering prelude (a pure elephc-PHP function over
    // a baked group/country table) only when the program references
    // DateTimeZone::listIdentifiers or timezone_identifiers_list, so other binaries
    // never carry the table. Runs after include resolution so usage inside includes
    // is detected, and before name resolution, which desugars both call forms to it.
    let phase_started = Instant::now();
    let ast = list_id_prelude::inject_if_used(ast);
    timings.record_since("list-id-prelude", phase_started);

    // Inject the image standard-library prelude (elephc_image externs + GD/Exif/
    // Imagick/Gmagick/Cairo surface, written in elephc-PHP) only when the program
    // references an image symbol, so non-image binaries never declare the
    // elephc_image externs or link the bridge. Runs after include resolution so
    // image usage inside includes is detected.
    let phase_started = Instant::now();
    let ast = crate::image_prelude::inject_if_used(ast, with_crates.contains("image"));
    timings.record_since("image-prelude", phase_started);

    let phase_started = Instant::now();
    let ast = web_prelude::inject_if_web(ast, web, php_version);
    timings.record_since("web-prelude", phase_started);

    // Pre-scan Composer `autoload.files` entries for globally-declared (non-namespaced) free
    // functions, INCLUDING ones nested inside `if (!function_exists('X')) { function X() {} }`
    // guards, before any name resolution runs. Each `autoload.files`/PSR-4 file is name-resolved
    // in isolation (`autoload::load_autoloaded_file`), so a namespaced caller in one file cannot
    // see a same-program global declared in a DIFFERENT file through its own per-file symbol
    // table; installing this set lets `name_resolver::symbols::Symbols::canonical_function`'s
    // global fallback see it anyway, mirroring the existing `PRELUDE_GLOBAL_FUNCTIONS` mechanism
    // but for the program's own Composer polyfills instead of elephc's built-in preludes. The
    // install spans BOTH the main name-resolution pass and `autoload::run` (every per-file
    // isolated resolve happens inside `autoload::run`), since a namespaced caller can live in the
    // main program, an `include`d file, or another autoloaded file.
    let phase_started = Instant::now();
    let known_composer_global_functions = autoload::scan_composer_global_functions(&autoload_registry);
    timings.record_since("composer-global-fn-scan", phase_started);

    // Both the main name-resolution pass and `autoload::run` (which name-resolves every spliced
    // file in isolation) run inside ONE install of `known_composer_global_functions`, so a
    // namespaced caller anywhere in the program — main, `include`d, or autoloaded — sees the same
    // fallback set regardless of which of these two passes resolves its call site.
    let phase_started = Instant::now();
    let autoload_result = name_resolver::with_known_composer_global_functions(
        known_composer_global_functions,
        || -> Result<(parser::ast::Program, Vec<errors::CompileWarning>), errors::CompileError> {
            let ast = name_resolver::resolve(ast)?;
            autoload::run(ast, &autoload_root, &autoload_registry)
        },
    );
    timings.record_since("name-resolve", phase_started);

    let phase_started = Instant::now();
    let ast = match autoload_result {
        Ok((resolved, autoload_warnings)) => {
            for warning in &autoload_warnings {
                errors::report_warning(warning);
            }
            resolved
        }
        Err(e) => {
            errors::report(&e);
            process::exit(1);
        }
    };
    timings.record_since("autoload-run", phase_started);

    // Hoist conditionally-declared functions (`if (!function_exists('X')) { function X(...) {...} }`
    // and other conditionally-nested declarations) to the top level so top-level function
    // collection registers them. Runs after autoload so `polyfill_prune` has already dropped its
    // provided-function/optional-helper guards, and after name resolution so nested bodies are
    // already fully qualified and can be moved verbatim.
    let phase_started = Instant::now();
    let ast = resolver::hoist_conditional_function_declarations(ast);
    timings.record_since("hoist-conditional-fns", phase_started);

    // Inject the var_export prelude (a pure elephc-PHP function) only when the program
    // references var_export and does not declare its own, so other binaries carry
    // nothing. Runs AFTER autoload::run and AFTER hoist_conditional_function_declarations
    // so the detection scan sees the fully-expanded program INCLUDING PSR-4 autoloaded
    // files (var_export usage inside autoloaded Symfony files is detected here, not just
    // usage in include-expanded files), and the injected `function var_export` declaration
    // is present before the type checker's function-discovery collects functions. Name
    // resolution of those calls is handled by the prelude-global fallback in
    // `name_resolver::canonical_prelude_global_function_name` (commit 25e24ba02), which
    // canonicalizes a bare namespaced `var_export(...)` call to the global `var_export`
    // during the main pass and during each autoloaded file's isolated name-resolution, so
    // the call resolves to this injected declaration even though injection now happens
    // after name resolution. The prelude's own internal builtins (str_replace, sprintf,
    // is_*, ...) are matched by `check_builtin` on their bare lowercase names, which the
    // prelude source already uses, so they need no name-resolution pass. The injected
    // function is a plain top-level FunctionDecl, so the earlier hoist pass does not
    // touch it (it runs before this injection) and the subsequent fold/check collect it.
    let phase_started = Instant::now();
    let ast = var_export_prelude::inject_if_used(ast);
    timings.record_since("var-export-prelude", phase_started);

    // Inject the register_shutdown_function prelude (a pure elephc-PHP callback registry) only
    // when the program references register_shutdown_function and does not declare its own.
    // Placed at the exact same pipeline stage as var_export_prelude for the same reasons: after
    // autoload::run + the conditional-function hoist (so PSR-4 autoloaded usage is detected too)
    // and before the checker's function discovery. `codegen_ir` calls the prelude's internal
    // `__elephc_run_shutdown_functions()` runner directly by symbol (see
    // `shutdown_prelude::RUN_SHUTDOWN_FUNCTIONS_NAME`) from the top-level epilogue and from
    // `exit()`/`die()` lowering, so this must run before EIR lowering — which it does, being this
    // early in the pipeline.
    let phase_started = Instant::now();
    let ast = shutdown_prelude::inject_if_used(ast);
    timings.record_since("shutdown-prelude", phase_started);

    let phase_started = Instant::now();
    let ast = optimize::fold_constants(ast);
    timings.record_since("opt-fold", phase_started);

    // Pre-checker FALSE-ONLY fold+prune of curated never-available PHP extension guards
    // (fastcgi_finish_request, litespeed_finish_request, igbinary_*, frankenphp_*, apcu_*,
    // opcache_*, xdebug_*) so a Composer runtime's `if (function_exists('fastcgi_finish_request'))
    // { fastcgi_finish_request(); }` (or an `extension_loaded('igbinary') ? igbinary_serialize(...)
    // : ...` ternary) never reaches the checker with a call to a name elephc cannot resolve. Reuses
    // `fold_function_existence`/`prune_constant_control_flow` exactly as the post-checker pass does
    // below, but with `FunctionExistenceSet::for_pre_check`, which only ever proves a curated
    // extension name absent and never true-folds (JURY ADDENDUM #1 in the shutdown/extension-fold
    // spec) — any name the program itself declares (a real polyfill, not just a guard) is excluded.
    // Placed immediately after `fold_constants` (magic constants/`ifdef` conditionals have already
    // been substituted well before this point) and before tree-shaking, which only reads `ast`.
    // The prune here MUST be the minimal pre-checker variant (`prune_dead_static_branches`): this
    // is the only prune that runs BEFORE the type checker, and the full
    // `prune_constant_control_flow` performs checker-observable drops — the
    // unreachable-trailing-statement drop lets a top-level `return` inlined from an included file
    // swallow the entire rest of the program, and the effect-free-`ExprStmt` removal deletes
    // statements the checker must still validate — silently exempting entry statements and
    // autoload-spliced code from type checking.
    let phase_started = Instant::now();
    let pre_check_extension_set = optimize::FunctionExistenceSet::for_pre_check(&ast);
    let ast = optimize::fold_function_existence(ast, &pre_check_extension_set);
    let ast = optimize::prune_dead_static_branches(ast);
    timings.record_since("opt-precheck-ext-fold", phase_started);

    // Tree-shaking (Stage 2), behind `--tree-shake`: harvest the structural skeleton and run the
    // reachability fixpoint over the fully-autoloaded, constant-folded program. The result is
    // intentionally discarded — later stages (checker/ir_lower pruning) will consume it. Stage 2
    // only optionally dumps it to STDERR when `ELEPHC_TREE_SHAKE_DUMP=1`, so `--tree-shake` off is
    // byte-identical to today and the flag on still reaches the same diagnostics/codegen.
    if tree_shake {
        let phase_started = Instant::now();
        let skeleton = tree_shake::harvest_skeleton(&ast);
        let reachable = tree_shake::compute_reachable(&ast, &skeleton);
        if std::env::var("ELEPHC_TREE_SHAKE_DUMP").as_deref() == Ok("1") {
            eprint!("{}", tree_shake::dump_reachable(&reachable));
        }
        timings.record_since("tree-shake", phase_started);
    }

    let phase_started = Instant::now();
    let mut check_result = match types::check_with_target(&ast, target) {
        Ok(result) => result,
        Err(e) => {
            errors::report(&e);
            process::exit(1);
        }
    };
    timings.record_since("typecheck", phase_started);
    for warning in &check_result.warnings {
        errors::report_warning(warning);
    }
    codegen::prepare_declared_name_order(
        &ast,
        &check_result.classes,
        &check_result.interfaces,
    );

    if !target.supports_current_backend() {
        eprintln!(
            "Target '{}' is recognized, but it is outside the current supported target matrix",
            target
        );
        process::exit(1);
    }

    let phase_started = Instant::now();
    let exported_functions = match exports::collect(&ast, &check_result.functions) {
        Ok(exports) => exports,
        Err(e) => {
            errors::report(&e.with_file(filename.to_string()));
            process::exit(1);
        }
    };
    timings.record_since("exports-scan", phase_started);
    if matches!(emit, Emit::Executable) && !exported_functions.is_empty() {
        let names: Vec<&str> = exported_functions.keys().map(String::as_str).collect();
        eprintln!(
            "warning: ignoring #[Export] on functions {:?} — --emit cdylib is required to expose them",
            names
        );
    }

    if check_only {
        timings.report();
        println!("Checked '{}'", filename);
        return;
    }

    let phase_started = Instant::now();
    let ast = optimize::propagate_constants(ast);
    timings.record_since("opt-prop", phase_started);

    // Fold closed-world class/interface/trait/enum existence checks on literal names to booleans
    // using the checked closed world, so `class_exists`-guarded blocks that reference absent
    // optional-dependency classes become constant control flow the following passes can prune.
    // The program-level fold covers top-level statements and function bodies; the method-body
    // fold covers class/enum methods, which EIR lowering reads from `check_result.method_decls`.
    let phase_started = Instant::now();
    let existence_sets =
        optimize::ClassExistenceSets::from_program_and_check_result(&ast, &check_result);
    let ast = optimize::fold_class_existence(ast, &existence_sets);
    optimize::fold_class_existence_in_method_bodies(&mut check_result, &existence_sets);
    timings.record_since("opt-class-exists", phase_started);

    // Fold closed-world `function_exists('X')` on a literal name to a boolean, so a `!function_exists`
    // guard around a builtin redefinition becomes constant control flow the following passes prune.
    // The fold is conservative about runtime load order: it folds unconditionally-available names
    // (builtin/extern/date-alias -> true, genuinely-absent -> false) but leaves checked user
    // functions for codegen, which keeps `function_exists` on an include-loaded function a runtime
    // check. Covers top-level/function bodies and, separately, class/enum method bodies EIR reads
    // from `check_result`.
    let phase_started = Instant::now();
    let function_existence_set =
        optimize::FunctionExistenceSet::from_check_result(&check_result);
    let ast = optimize::fold_function_existence(ast, &function_existence_set);
    optimize::fold_function_existence_in_method_bodies(&mut check_result, &function_existence_set);
    timings.record_since("opt-func-exists", phase_started);

    // Coerce literal string-callable arguments (`'someFunc'`) at `callable`-typed
    // regular-parameter positions into their first-class-callable AST equivalent, so a call
    // like `register_shutdown_function('someFunc')` reaches EIR lowering as the same node shape
    // as writing `someFunc(...)` explicitly. The type checker (above, via `check_with_target`)
    // already ACCEPTED this coercion on an ephemeral copy of the call's arguments — this pass
    // performs the equivalent rewrite on the real AST that `ir_lower` will actually walk. See
    // `crate::optimize::callable_coercion` module docs for the exact (narrow, documented) scope.
    let phase_started = Instant::now();
    let callable_coercion_set = optimize::CallableCoercionSet::from_check_result(&check_result);
    let ast = optimize::coerce_callable_string_args(ast, &callable_coercion_set);
    optimize::coerce_callable_string_args_in_method_bodies(&mut check_result, &callable_coercion_set);
    timings.record_since("opt-callable-coercion", phase_started);

    let phase_started = Instant::now();
    let ast = optimize::prune_constant_control_flow(ast);
    timings.record_since("opt-post", phase_started);

    let phase_started = Instant::now();
    let ast = optimize::normalize_control_flow(ast);
    timings.record_since("opt-norm", phase_started);

    let phase_started = Instant::now();
    let ast = optimize::eliminate_dead_code(ast);
    timings.record_since("dce", phase_started);

    if emit_ir {
        let phase_started = Instant::now();
        let mut module = match ir_lower::lower_program_with_source_path_and_web(
            &ast,
            &check_result,
            target,
            Path::new(filename),
            web,
            &class_source_files,
            &function_source_files,
        ) {
            Ok(module) => module,
            Err(err) => {
                eprintln!("EIR lowering error: {}", err);
                process::exit(1);
            }
        };
        timings.record_since("ir-lower", phase_started);

        let phase_started = Instant::now();
        if ir_opt {
            ir_passes::optimize_module(&mut module);
        }
        timings.record_since("ir-opt", phase_started);

        let phase_started = Instant::now();
        let text = ir::print_module(&module);
        timings.record_since("ir-print", phase_started);
        timings.report();
        print!("{}", text);
        return;
    }

    let phase_started = Instant::now();
    let mut ir_module = match ir_lower::lower_program_with_source_path_and_web(
        &ast,
        &check_result,
        target,
        Path::new(filename),
        web,
        &class_source_files,
        &function_source_files,
    ) {
        Ok(module) => module,
        Err(err) => {
            eprintln!("EIR lowering error: {}", err);
            process::exit(1);
        }
    };
    timings.record_since("ir-lower", phase_started);

    let phase_started = Instant::now();
    if ir_opt {
        ir_passes::optimize_module(&mut ir_module);
    }
    timings.record_since("ir-opt", phase_started);

    let mut runtime_features = ir_module.required_runtime_features;
    // `--web` selects the output-capture variant of `__rt_stdout_write`. This is the
    // sole driver of the web runtime feature: it is CLI-driven, not derived from the
    // program, so the runtime cache (keyed on the generated assembly hash) keeps the
    // web and non-web runtime objects distinct automatically.
    runtime_features.web = web;

    if web && !extra_link_libs.iter().any(|lib| lib == "elephc_web") {
        extra_link_libs.push("elephc_web".to_string());
    }

    // `--with-<crate>` force-links each named bridge staticlib (whole-archived,
    // via `forced_bridge_libs`, so it is not dead-stripped) regardless of feature
    // auto-detection. Crates with a PHP-surface prelude (pdo/tz/image) also had
    // that prelude force-injected above, so their classes/functions are available.
    let mut forced_bridge_libs: Vec<String> = Vec::new();
    for flag in &with_crates {
        if let Some(lib) = linker::bridge_lib_for_flag(flag) {
            if !extra_link_libs.iter().any(|l| l == lib) {
                extra_link_libs.push(lib.to_string());
            }
            forced_bridge_libs.push(lib.to_string());
        }
    }

    let requires_elephc_tls = extra_link_libs.iter().any(|lib| lib == "elephc_tls")
        || check_result
            .required_libraries
            .iter()
            .any(|lib| lib == "elephc_tls");

    let phase_started = Instant::now();
    let runtime_pic = matches!(emit, Emit::Cdylib);
    let runtime_object = match runtime_cache::prepare_runtime_object(heap_size, target, runtime_features, runtime_pic) {
        Ok(runtime_object) => runtime_object,
        Err(err) => {
            eprintln!("Runtime cache error: {}", err);
            process::exit(1);
        }
    };
    timings.record_since("runtime-cache", phase_started);
    timings.note(format!("runtime-cache {}", runtime_object.status.as_str()));

    let phase_started = Instant::now();
    let user_asm = match codegen::generate_user_asm_from_ir_with_options(
        &ir_module,
        gc_stats,
        heap_debug,
        requires_elephc_tls,
        emit,
        &exported_functions,
        regalloc_linear,
        web,
    ) {
        Ok(asm) => asm,
        Err(err) => {
            eprintln!("EIR backend error: {}", err);
            process::exit(1);
        }
    };
    let user_asm = if emit_debug_info {
        debug_info::inject_line_directives(&user_asm, filename, target.platform)
    } else {
        user_asm
    };
    timings.record_since("codegen", phase_started);

    for lib in &check_result.required_libraries {
        if !extra_link_libs.contains(lib) {
            extra_link_libs.push(lib.clone());
        }
    }
    for lib in codegen::required_libraries_for_runtime_features(runtime_features) {
        if !extra_link_libs.contains(&lib) {
            extra_link_libs.push(lib);
        }
    }

    let phase_started = Instant::now();
    if let Err(e) = fs::write(&output_paths.asm, &user_asm) {
        eprintln!("Error writing '{}': {}", output_paths.asm.display(), e);
        process::exit(1);
    }
    timings.record_since("write-asm", phase_started);

    if emit_source_map {
        let phase_started = Instant::now();
        if let Err(err) =
            source_map::write_source_map(
                &user_asm,
                Path::new(filename),
                &output_paths.asm,
                &output_paths.source_map,
            )
        {
            eprintln!("Source map error: {}", err);
            process::exit(1);
        }
        timings.record_since("source-map", phase_started);
    }

    if emit_asm {
        timings.report();
        println!(
            "Emitted assembly '{}' -> '{}'",
            filename,
            output_paths.asm.display()
        );
        return;
    }

    let phase_started = Instant::now();
    linker::assemble(target, &output_paths.asm, &output_paths.obj);
    timings.record_since("assemble", phase_started);

    let phase_started = Instant::now();
    linker::link(
        target,
        emit,
        &output_paths.bin,
        &output_paths.obj,
        &runtime_object.path,
        &extra_link_libs,
        &extra_link_paths,
        &extra_frameworks,
        &forced_bridge_libs,
    );
    timings.record_since("link", phase_started);

    // With --debug-info the DWARF line tables must be preserved past object
    // cleanup: on macOS `dsymutil` bakes them into a .dSYM while the object
    // still exists; if that fails the object is kept so debuggers can follow
    // the binary's debug map to it.
    let keep_obj_for_debug =
        emit_debug_info && !linker::bake_debug_info(target, &output_paths.bin);
    if !keep_obj_for_debug {
        let _ = fs::remove_file(&output_paths.obj);
    }

    timings.report();
    println!("Compiled '{}' -> '{}'", filename, output_paths.bin.display());
}

/// Computes output paths for .s (assembly), .o (object), binary, and .map (source map) files
/// derived from the input filename.
///
/// Executable mode produces `<stem>` (no extension). Cdylib mode produces
/// `lib<stem>.so` (Linux) or `lib<stem>.dylib` (macOS), matching the conventional
/// shared-library naming that `dlopen(3)` and linker `-l` flags expect.
fn output_paths(filename: &str, target: Target, emit: Emit) -> OutputPaths {
    let path = Path::new(filename);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let parent = path.parent().unwrap_or(Path::new("."));
    let bin_name = match emit {
        Emit::Executable => stem.to_string(),
        Emit::Cdylib => match target.platform {
            Platform::MacOS => format!("lib{}.dylib", stem),
            Platform::Linux => format!("lib{}.so", stem),
            Platform::Windows => panic!("Windows target is not yet supported (see issue #379)"),
        },
    };
    OutputPaths {
        asm: parent.join(format!("{}.s", stem)),
        obj: parent.join(format!("{}.o", stem)),
        bin: parent.join(bin_name),
        source_map: parent.join(format!("{}.map", stem)),
    }
}
