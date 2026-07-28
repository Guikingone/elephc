//! Purpose:
//! Provides the binary entry point for the elephc compiler.
//! Wires CLI parsing to the ordered compile pipeline without owning compiler logic.
//!
//! Called from:
//! - The operating system when running the `elephc` executable.
//!
//! Key details:
//! - Keep startup thin so CLI validation and pipeline behavior stay in dedicated modules.

mod autoload;
mod builtins;
mod cli;
mod codegen;
mod codegen_support;
mod conditional;
mod errors;
mod eval_aot;
mod exports;
mod hash_prelude;
mod image_prelude;
mod intrinsics;
#[allow(dead_code, unused_imports)]
mod ir;
#[allow(dead_code, unused_imports)]
mod ir_lower;
#[allow(dead_code, unused_imports)]
mod ir_passes;
mod lexer;
mod linker;
mod list_id_prelude;
mod magic_constants;
mod name_resolver;
mod names;
mod opcache;
mod opcache_prelude;
mod optimize;
mod parser;
mod pdo_prelude;
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
mod termination;
mod timings;
mod types;
mod tz_prelude;
mod var_export_prelude;
mod version_prelude;
mod web_prelude;

/// Entry point for the `elephc` binary.
///
/// Collects command-line arguments, parses them into a `Config`, and delegates
/// to the compile pipeline. Exits via `std::process::exit` if compilation fails
/// (the pipeline handles fatal error reporting internally).
///
/// # Inputs
/// - `std::env::args()`: OS-provided arguments, where `args[0]` is the program name.
///
/// # Outputs
/// - Returns `()` on successful compilation (pipeline handles output binary creation).
/// - Never returns on fatal error (calls `std::process::exit` internally).
///
/// # Side effects
/// - Reads source files and writes the compiled binary alongside the source.
/// - Emits warnings/errors to stderr, including the OPcache `--ini` quantity diagnostics
///   ([`emit_ini_override_warnings`]).
/// - May create temporary files during assembly and linking.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if cli::wants_mascotte(&args) {
        cli::print_mascotte();
    }
    let config = cli::parse_args(&args);
    emit_ini_override_warnings(&config);
    pipeline::compile(config);
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
