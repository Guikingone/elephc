//! Purpose:
//! Lowers a checked and optimized PHP AST into EIR for the active backend.
//! Owns the AST-to-IR semantic boundary before validation and EIR codegen.
//!
//! Called from:
//! - `crate::pipeline::compile()` before optimization, register allocation, and codegen.
//!
//! Key details:
//! - Lowering preserves PHP source evaluation order by walking the AST in
//!   source order and emitting high-level EIR operations.
//! - EIR is the only production backend; unsupported lowering must fail explicitly.

mod builtin_datetime;
mod context;
mod effects_lookup;
mod expr;
mod fibers;
mod function;
mod ownership;
mod program;
mod reflection;
mod stmt;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use crate::codegen::platform::Target;
use crate::ir::{Module, ValidationError};
use crate::parser::ast::Program;
use crate::types::CheckResult;

/// Lowers `program` into an EIR module for `target`.
///
/// `web` is the CLI `--web` flag; it is stored on the returned module (see
/// `crate::ir::Module::web`) so lowering can gate request-superglobal
/// (`$_SERVER`/`$_SESSION`/…) type seeding on it, mirroring the `web` gate
/// `codegen_ir::block_emit::emit_module` already applies to `.comm` storage.
///
/// `class_source_files`/`function_source_files` (see
/// `lower_program_with_source_path_and_web`) are the case-folded-name ->
/// declaring-file maps produced by `crate::resolver::scan_reflection_source_files`
/// for the entry file; they back `ReflectionClass`/`ReflectionFunction::getFileName()`.
/// This wrapper passes empty maps (e.g. for most tests).
pub fn lower_program(
    program: &Program,
    check_result: &CheckResult,
    target: Target,
    web: bool,
) -> Result<Module, LoweringError> {
    static EMPTY: std::sync::OnceLock<HashMap<String, String>> = std::sync::OnceLock::new();
    let empty = EMPTY.get_or_init(HashMap::new);
    program::lower(program, check_result, target, None, web, empty, empty)
}

/// Lowers `program` into an EIR module and records the main PHP source path.
pub fn lower_program_with_source_path(
    program: &Program,
    check_result: &CheckResult,
    target: Target,
    source_path: &Path,
) -> Result<Module, LoweringError> {
    static EMPTY: std::sync::OnceLock<HashMap<String, String>> = std::sync::OnceLock::new();
    let empty = EMPTY.get_or_init(HashMap::new);
    program::lower(program, check_result, target, Some(source_path), false, empty, empty)
}

/// Lowers `program` into EIR while retaining source-path, web-mode, and
/// Reflection `getFileName()` source-file metadata.
pub fn lower_program_with_source_path_and_web(
    program: &Program,
    check_result: &CheckResult,
    target: Target,
    source_path: &Path,
    web: bool,
    class_source_files: &HashMap<String, String>,
    function_source_files: &HashMap<String, String>,
) -> Result<Module, LoweringError> {
    program::lower(
        program,
        check_result,
        target,
        Some(source_path),
        web,
        class_source_files,
        function_source_files,
    )
}

/// Error produced while building or validating EIR.
#[derive(Debug)]
pub enum LoweringError {
    Validation(ValidationError),
}

impl fmt::Display for LoweringError {
    /// Formats the lowering error for CLI diagnostics.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoweringError::Validation(err) => write!(f, "EIR validation failed: {:?}", err),
        }
    }
}

impl From<ValidationError> for LoweringError {
    /// Converts an EIR validation error into a lowering error.
    fn from(value: ValidationError) -> Self {
        LoweringError::Validation(value)
    }
}
