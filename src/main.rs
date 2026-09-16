//! Purpose:
//! Provides the binary entry point for the compiler and native dependency commands.
//! Wires top-level dispatch to the appropriate orchestration layer.
//!
//! Called from:
//! - The operating system when running the `elephc` executable.
//!
//! Key details:
//! - Keep startup thin so CLI validation and pipeline behavior stay in dedicated modules.

mod autoload;
mod brand;
mod builtins;
mod cli;
mod codegen;
mod codegen_support;
mod conditional;
mod curl_prelude;
mod errors;
mod eval_aot;
mod exports;
mod func_args;
mod global_decls;
mod hash_prelude;
mod image_prelude;
mod intrinsics;
#[allow(dead_code, unused_imports)]
mod ir;
#[allow(dead_code, unused_imports)]
mod ir_lower;
#[allow(dead_code, unused_imports)]
mod ir_passes;
#[allow(dead_code)]
mod link_plan;
mod link_planning;
mod linker;
mod lexer;
mod list_id_prelude;
mod magic_constants;
mod name_resolver;
#[allow(dead_code, unused_imports)]
mod native_deps;
mod names;
mod numeric_string;
mod object_cast_prelude;
mod opcache;
mod opcache_prelude;
mod optimize;
mod otlp;
mod parser;
mod php_version;
mod mysqli_prelude;
mod pdo_prelude;
mod monitor;
mod call_graph;
mod php_profile;
mod pprof_encode;
mod prelude_prune;
mod probe_key;
mod pipeline;
mod progress;
mod resolver;
mod runtime_cache;
mod debug_info;
mod source;
mod source_map;
mod span;
mod strict_php;
mod string_bytes;
mod superglobals;
#[allow(dead_code)]
mod synthetic_class;
mod termination;
mod timings;
mod types;
mod tz_prelude;
mod xml_prelude;
mod var_export_prelude;
mod version_prelude;
mod web_prelude;

/// Entry point for the `elephc` binary.
///
/// Collects command-line arguments, parses the top-level command, and delegates
/// to either compilation or explicit native-dependency orchestration.
///
/// # Inputs
/// - `std::env::args()`: OS-provided arguments, where `args[0]` is the program name.
///
/// # Outputs
/// - Returns `()` when the selected command succeeds without an explicit exit.
/// - Never returns on fatal errors or unhealthy native diagnostics.
///
/// # Side effects
/// - Compile commands read source files and write outputs alongside the source.
/// - Mutating native commands may update project files and the durable native cache.
/// - Emits warnings/errors to stderr, including OPcache `--ini` quantity diagnostics
///   ([`emit_ini_override_warnings`]) for compile commands.
/// - May create temporary files during assembly and linking.
fn main() {
    run_on_compiler_stack(main_inner)
}

/// The stack every compiler thread gets.
///
/// Sized against the limit the compiler ALREADY diagnoses. `MAX_COMPILER_NESTING` lets source
/// nest 1024 levels deep, and each level costs one frame in every recursive AST pass -- the
/// parser, the constant folder, the magic-constant walker, the checker, the optimizer's
/// rewriters, EIR lowering. The default 8 MiB main stack runs out around 140 levels, so
/// `$a = [[[…1…]]]` at 200 aborted the process with `has overflowed its stack` instead of
/// reporting the diagnostic written for exactly that input (issue #686).
///
/// A thread rather than a per-pass guard because the passes are many and the list grows: one
/// place to size, and a pass added later inherits it. PHP itself compiles these depths, so a
/// diagnostic below 1024 would reject valid PHP rather than protect anything.
const COMPILER_STACK_BYTES: usize = 256 * 1024 * 1024;

/// Runs `body` on a thread with [`COMPILER_STACK_BYTES`] of stack, propagating its panic.
///
/// A stack this size is RESERVED, not committed: the pages are only faulted in as the recursion
/// actually reaches them, so an ordinary compile pays for the depth it uses and nothing more.
fn run_on_compiler_stack(body: fn()) {
    let worker = std::thread::Builder::new()
        .name("elephc-compiler".to_string())
        .stack_size(COMPILER_STACK_BYTES)
        .spawn(body);
    match worker {
        Ok(handle) => {
            if let Err(panic) = handle.join() {
                // The thread already printed the panic message; resume it here so the process
                // exits the way it would have without the extra thread.
                std::panic::resume_unwind(panic);
            }
        }
        // A machine that cannot spawn the thread still has its own stack; running inline keeps
        // the compiler usable there instead of failing before it starts.
        Err(_) => body(),
    }
}

/// The real entry point, running on the compiler stack established by [`run_on_compiler_stack`].
fn main_inner() {
    let args: Vec<String> = std::env::args().collect();
    if cli::wants_mascotte(&args) {
        cli::print_mascotte();
    }
    match cli::parse_args(&args) {
        cli::Command::Compile(config) => {
            emit_ini_override_warnings(&config);
            pipeline::compile(config);
        }
        cli::Command::Native(command) => run_native(command),
        cli::Command::Monitor(command) => std::process::exit(monitor::run(command)),
    }
}

/// Executes a parsed native command and maps its captured output to process streams/status.
fn run_native(command: native_deps::NativeCommand) {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            eprintln!("failed to read current directory: {error}");
            std::process::exit(1);
        }
    };
    match native_deps::run_native_command(&command, &cwd) {
        Ok(output) => {
            print!("{}", output.stdout);
            if output.exit_code != 0 {
                std::process::exit(output.exit_code);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

/// Prints the startup diagnostics reference PHP would emit for the `--ini` overrides this
/// compile carries, to stderr, before the pipeline runs.
///
/// Reference PHP emits these while REGISTERING the INI entries at startup — a
/// `Warning: Invalid "opcache.max_file_size" setting. Invalid quantity "12abc": unknown
/// multiplier "c", interpreting as "12" for backwards compatibility in Unknown on line 0` for
/// `php -d opcache.max_file_size=12abc`. For elephc the compile IS the registration (the
/// directive values are baked into the binary), so this is where the faithful analogue belongs
/// and the only point at which it is actionable. The value is still STORED either way — see
/// `crate::opcache::directives::parse_ini_quantity` — so without this the misread is silent.
///
/// The `in Unknown on line 0` tail is dropped: it names reference PHP's INI-file position, and
/// elephc's source of the value is a command-line flag, which the compiler's own stderr voice
/// already implies. Nothing is emitted when there are no `--ini` overrides, so the default
/// compile path is byte-identical on stderr.
fn emit_ini_override_warnings(config: &cli::CliConfig) {
    for warning in opcache::directives::ini_override_warnings(
        config.php_version.version_id(),
        &config.ini_overrides,
    ) {
        eprintln!("Warning: {warning}");
    }
}
