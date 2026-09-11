//! Command-line dispatch shared through the compiler library.
//! Preserves startup diagnostics, native/monitor commands and the bounded compiler stack.

use crate::{cli, monitor, native_deps, opcache, pipeline};

#[cfg(test)]
mod tests {
    #[test]
    fn cli_executable_delegates_without_redeclaring_compiler_modules() {
        let executable = include_str!("main.rs");
        assert!(executable.contains("elephc::run_cli()"));
        assert!(!executable.lines().any(|line| line.trim_start().starts_with("mod ")),
            "compiler modules belong in the library, not a second binary crate copy");
    }
}

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
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    if cli::wants_mascotte(&args) {
        cli::print_mascotte();
    }
    match cli::parse_args(&args) {
        cli::Command::Compile(config) => {
            emit_ini_override_warnings(&config);
            run_compile_with_stack(config);
        }
        cli::Command::Native(command) => run_native(command),
        cli::Command::Monitor(command) => std::process::exit(monitor::run(command)),
    }
}

/// Runs one compilation on a dedicated stack sized for deep PHP source graphs.
///
/// The operating-system main thread has a small fixed stack on macOS. Frontend
/// traversals legitimately recurse through deeply nested declarations and include
/// graphs, so the compiler owns a bounded 64 MiB worker stack for every compile.
fn run_compile_with_stack(config: cli::CliConfig) {
    const COMPILE_STACK_BYTES: usize = 64 * 1024 * 1024;
    let worker = std::thread::Builder::new()
        .name("elephc-compile".to_string())
        .stack_size(COMPILE_STACK_BYTES)
        .spawn(move || pipeline::compile(config))
        .expect("failed to create compiler worker thread");
    if let Err(payload) = worker.join() {
        std::panic::resume_unwind(payload);
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
