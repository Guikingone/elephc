//! Purpose:
//! Gives eval code the superglobals PHP's CLI SAPI has populated when the fragment names one
//! that the compiled program never created.
//!
//! Called from:
//! - `crate::interpreter::execute_program_outcome_with_context()` and eval `include`.
//!
//! Key details:
//! - Seeding follows PHP's `auto_globals_jit`: only the superglobals a fragment's code names
//!   are created, when that fragment starts, and only when the scope they resolve through lacks
//!   them, so a value the compiled program synchronized in is never replaced.
//! - Contents mirror the compiler's `superglobals::seed_cli_populated_superglobals`: `$_ENV` is
//!   the environment, `$_SERVER` the environment plus the nine CLI keys, and `$_GET`, `$_POST`,
//!   `$_COOKIE`, `$_FILES`, `$_REQUEST` empty arrays.

use super::builtins::collection_builder::EvalArrayBuilder;
use super::*;
use std::os::unix::ffi::OsStrExt;

/// Seeds the CLI-populated superglobals `program` names that are not yet visible.
///
/// Inside an executing `eval()` the values land in the global scope (see `scope_cells`), which
/// in the top-level scope is the eval scope itself.
pub(in crate::interpreter) fn seed_named_cli_superglobals(
    program: &EvalProgram,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    for name in program.cli_superglobals() {
        if visible_scope_cell(context, scope, name).is_some() {
            continue;
        }
        let value = eval_cli_superglobal_value(name, values)?;
        for replaced in set_owned_scope_cell(context, scope, (*name).to_string(), value, values)? {
            eval_release_value(context, values, replaced)?;
        }
    }
    Ok(())
}

/// Builds the value PHP's CLI SAPI gives one superglobal.
fn eval_cli_superglobal_value(
    name: &str,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match name {
        "_ENV" => eval_getenv_all_result(values),
        "_SERVER" => eval_cli_server_value(values),
        _ => values.array_new(0),
    }
}

/// Builds `$_SERVER`: the environment first, then the nine keys PHP's CLI SAPI adds on top.
///
/// The four path-shaped keys name what was invoked, `argv[0]`, and `DOCUMENT_ROOT` is empty,
/// exactly as in the compiled program's seed; `argv`/`argc` are the process arguments. Every
/// temporary key and value is released once the array retains it.
fn eval_cli_server_value(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let arguments = std::env::args_os()
        .map(|argument| argument.as_bytes().to_vec())
        .collect::<Vec<_>>();
    let argc = i64::try_from(arguments.len()).map_err(|_| EvalStatus::RuntimeFatal)?;
    let invoked = arguments.first().cloned().unwrap_or_default();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let environment = eval_getenv_all_result(values)?;
    let mut server = EvalArrayBuilder::from_owned(values, environment);
    for key in ["PHP_SELF", "SCRIPT_NAME", "SCRIPT_FILENAME", "PATH_TRANSLATED"] {
        server.string(key, |values| values.string_bytes_value(&invoked))?;
    }
    server.string("DOCUMENT_ROOT", |values| values.string(""))?;
    server.string("REQUEST_TIME", |values| {
        values.int(i64::try_from(now.as_secs()).unwrap_or(i64::MAX))
    })?;
    server.string("REQUEST_TIME_FLOAT", |values| values.float(now.as_secs_f64()))?;
    server.string("argv", |values| {
        let mut argv = EvalArrayBuilder::indexed(values, arguments.len())?;
        for (index, argument) in arguments.iter().enumerate() {
            argv.index(index, |values| values.string_bytes_value(argument))?;
        }
        Ok(argv.finish())
    })?;
    server.string("argc", |values| values.int(argc))?;
    Ok(server.finish())
}
