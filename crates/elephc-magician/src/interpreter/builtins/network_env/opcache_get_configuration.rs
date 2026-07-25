//! Purpose:
//! Eval-interpreter implementation of `opcache_get_configuration()`. Builds the
//! `['directives' => [...], 'version' => [...], 'blacklist' => []]` array from the
//! same version-keyed OPcache directive matrix the native prelude renders, so the
//! two surfaces never drift.
//!
//! Called from:
//! - `crate::interpreter::expressions::calls::eval_call` (direct dispatch).
//! - `crate::interpreter::builtins::registry::dispatch::eval_builtin_with_values`
//!   (dynamic-callable / by-values dispatch).
//! - `crate::interpreter::builtins::symbols::function_exists` (existence probe).
//!
//! Key details:
//! - `opcache_get_configuration` is prelude-provided on the native side (a real PHP
//!   function), NOT a checker catalog builtin, so it must NOT be a PHP-visible eval
//!   builtin either (that would break `builtin_parity_tests`, which require the two
//!   PHP-visible builtin sets to agree). It is therefore dispatched as a plain
//!   runtime handler and made visible to `function_exists` through a small allowlist,
//!   exactly as the procedural date/time aliases are.
//! - The directive table is shared verbatim from `src/opcache/directives.rs` via a
//!   `#[path]` include (mirroring how `time/aliases.rs` shares
//!   `src/list_id_prelude/table.rs`), so there is a single source of truth across the
//!   two crates with no duplication.
//! - The eval interpreter has no compile-target selector, so it reports the newest
//!   maintained profile (PHP 8.5), matching the native default target.

use super::*;

#[path = "../../../../../../src/opcache/directives.rs"]
mod opcache_directive_table;

use opcache_directive_table::{
    opcache_directives, opcache_version_string, DirectiveValue, OPCACHE_PRODUCT_NAME,
};

/// The `PHP_VERSION_ID` the eval interpreter reports for OPcache configuration.
/// Fixed to the newest maintained profile (8.5), matching the native default target.
const EVAL_OPCACHE_PHP_VERSION_ID: u32 = 80500;

/// Returns whether `name` (already lowercased and unqualified) is the OPcache
/// configuration function, so `function_exists` reports it as existing even though
/// it is not a PHP-visible eval builtin.
pub(in crate::interpreter) fn eval_opcache_configuration_function_exists(name: &str) -> bool {
    name == "opcache_get_configuration"
}

/// Evaluates a direct `opcache_get_configuration()` call from an eval fragment.
pub(in crate::interpreter) fn eval_opcache_get_configuration_call(
    args: &[EvalCallArg],
    _context: &mut ElephcEvalContext,
    _scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if !args.is_empty() {
        return Err(EvalStatus::RuntimeFatal);
    }
    eval_opcache_get_configuration_result(values)
}

/// Builds the `opcache_get_configuration()` return array as runtime cells.
pub(in crate::interpreter) fn eval_opcache_get_configuration_result(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let directives = build_directives(values)?;
    let version = build_version(values)?;
    let blacklist = values.array_new(0)?;

    let mut configuration = values.assoc_new(3)?;
    let directives_key = values.string("directives")?;
    configuration = values.array_set(configuration, directives_key, directives)?;
    let version_key = values.string("version")?;
    configuration = values.array_set(configuration, version_key, version)?;
    let blacklist_key = values.string("blacklist")?;
    configuration = values.array_set(configuration, blacklist_key, blacklist)?;
    Ok(configuration)
}

/// Builds the `'directives'` sub-array from the shared directive matrix.
fn build_directives(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let entries = opcache_directives(EVAL_OPCACHE_PHP_VERSION_ID);
    let mut directives = values.assoc_new(entries.len())?;
    for (name, value) in &entries {
        let key = values.string(name)?;
        let value = build_directive_value(value, values)?;
        directives = values.array_set(directives, key, value)?;
    }
    Ok(directives)
}

/// Materializes one typed directive value as a runtime cell.
fn build_directive_value(
    value: &DirectiveValue,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match value {
        DirectiveValue::Bool(boolean) => values.bool_value(*boolean),
        DirectiveValue::Int(integer) => values.int(*integer),
        DirectiveValue::Float(float) => values.float(*float),
        DirectiveValue::Str(string) => values.string(string),
    }
}

/// Builds the `'version'` sub-array (`version` + `opcache_product_name`).
fn build_version(values: &mut impl RuntimeValueOps) -> Result<RuntimeCellHandle, EvalStatus> {
    let mut version = values.assoc_new(2)?;
    let version_key = values.string("version")?;
    let version_value = values.string(opcache_version_string(EVAL_OPCACHE_PHP_VERSION_ID))?;
    version = values.array_set(version, version_key, version_value)?;
    let product_key = values.string("opcache_product_name")?;
    let product_value = values.string(OPCACHE_PRODUCT_NAME)?;
    version = values.array_set(version, product_key, product_value)?;
    Ok(version)
}
