//! Purpose:
//! Lowers a checked and optimized PHP AST into EIR for diagnostics and the
//! future EIR backend track.
//!
//! Called from:
//! - `crate::pipeline::compile()` when `--emit-ir` is requested.
//!
//! Key details:
//! - Lowering preserves PHP source evaluation order by walking the AST in
//!   source order and emitting high-level EIR operations.
//! - The legacy AST-to-ASM backend remains the production path.

mod builtin_datetime;
mod context;
mod effects_lookup;
mod expr;
mod fibers;
mod function;
mod ownership;
mod program;
mod stmt;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fmt;

use crate::codegen::platform::Target;
use crate::ir::{Module, ValidationError};
use crate::parser::ast::Program;
use crate::types::CheckResult;

/// Lowers `program` into an EIR module for `target`.
///
/// `class_source_files`/`function_source_files` are the case-folded-name -> declaring-file maps
/// produced by `crate::pipeline::scan_reflection_source_files` for the entry file; they back
/// `ReflectionClass`/`ReflectionFunction::getFileName()` (see
/// `crate::codegen_ir::lower_inst::objects::reflection`). Pass empty maps when this feature is
/// not needed (e.g. most tests).
pub fn lower_program(
    program: &Program,
    check_result: &CheckResult,
    target: Target,
    class_source_files: &HashMap<String, String>,
    function_source_files: &HashMap<String, String>,
) -> Result<Module, LoweringError> {
    program::lower(program, check_result, target, class_source_files, function_source_files)
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
