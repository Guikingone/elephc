//! Purpose:
//! Coordinates conservative whole-program declaration reachability after AST DCE.
//! Owns the public pruning options and the AST/metadata reconciliation boundary.
//!
//! Called from:
//! - `crate::pipeline::compile()` and codegen fixture pipelines before EIR lowering.
//!
//! Key details:
//! - The pass removes declarations only after a fixed-point reachability proof.
//! - The AST and `CheckResult` are pruned together so lowering cannot resurrect dead methods.

use crate::fast_hash::FastSet as HashSet;

use crate::parser::ast::Program;
use crate::types::CheckResult;

pub mod graph;
pub mod inventory;
mod prune;
mod reconcile;
pub mod usage;

pub use inventory::PreludeInventory;

/// Inputs that add compiler-owned roots to the declaration graph.
pub struct PruneOptions<'a> {
    pub inventory: &'a PreludeInventory,
    // These two stay STD-hashed on purpose. They are inputs handed in by the pipeline and are read
    // a handful of times, not keys in the fixed point's hot path, so there is nothing to gain by
    // making every caller build them with the fast hasher — and this is a public signature.
    pub forced_groups: &'a std::collections::HashSet<String>,
    pub exported_functions: &'a std::collections::HashSet<String>,
    pub eval_forced: bool,
}

/// Removes unreachable declarations and reconciles all checker metadata consumed by EIR lowering.
pub fn prune_unreachable_declarations(
    program: Program,
    check_result: &mut CheckResult,
    options: PruneOptions<'_>,
) -> Program {
    let original_builtin_libraries = usage::scan_program(&program).required_libraries;
    let reachability = graph::compute(&program, check_result, &options);
    let declaration_index = graph::DeclarationIndex::build(&program, check_result);
    let program = prune::program(program, &reachability);
    let remaining_builtin_libraries = usage::scan_program(&program).required_libraries;
    reconcile::check_result(
        check_result,
        &reachability,
        &declaration_index,
        &original_builtin_libraries,
        &remaining_builtin_libraries,
    );
    program
}

#[cfg(test)]
mod tests;
