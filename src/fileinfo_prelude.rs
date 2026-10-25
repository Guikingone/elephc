//! Purpose:
//! Injects PHP's six fileinfo functions and finfo class over the optional native bridge.
//!
//! Called from:
//! - The compilation pipeline and EIR/codegen test harness before name resolution.
//!
//! Key details:
//! - Synthetic declarations use the ordinary EIR, object and extern ABI paths on every target.
//! - Opaque dynamic eval can opt in with --with-fileinfo.

mod build;
mod detect;
mod streams;
mod wrappers;
mod pending;
use crate::parser::ast::Program;

/// Builds the class, PHP functions and bridge extern declarations.
pub(crate) fn fileinfo_declarations() -> Program { build::declarations() }

/// Prepends the fileinfo surface only when referenced or explicitly forced.
pub fn inject_if_used(program: Program, force: bool, inventory: &mut crate::optimize::reachability::PreludeInventory) -> Program {
    crate::compiler_stack::with_compiler_stack(|| {
        if !force && !detect::program_uses_fileinfo(&program) { return program; }
        let mut declarations = fileinfo_declarations();
        inventory.record_program("fileinfo", &declarations);
        declarations.extend(program);
        declarations
    })
}
