//! Purpose:
//! Orchestrates AST-to-EIR lowering for a complete checked program.
//!
//! Called from:
//! - `crate::ir_lower::lower_program()`.
//!
//! Key details:
//! - Declaration bodies are lowered before synthetic `main`; declaration
//!   statements themselves are no-ops inside `main`.
//! - The module is validated before it is returned to CLI/test callers.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

use crate::codegen::platform::Target;
use crate::codegen::RuntimeFeatures;
use crate::intrinsics::IntrinsicCall;
use crate::ir::{
    validate_function, validate_module, ExternDecl, ExternParamDecl, Function, Immediate, IrType,
    InstId, LocalKind, Module, Op, TraitMethodInfo, ValidationError, ValueDef, ValueId,
};
use crate::ir_lower::{body_contains_eval_call, builtin_datetime, function, LoweringError};
use crate::names::php_symbol_key;
use crate::parser::ast::{
    ClassMethod, Expr, ExprKind, Program, StaticReceiver, Stmt, StmtKind, Visibility,
};
use crate::types::{CheckResult, ClassInfo, FunctionSig, InterfaceInfo, PhpType};

mod metadata;
mod runtime_features;
mod eval_aot;
/// Read by `builtin_datetime`, which decides whether a module's eval fragments can reach the
/// date alias surface — a question only the fragment text can answer.
/// Read by `builtin_datetime`, which asks whether a module's eval fragments can reach the date
/// alias surface. `eval_literal_call_requires_bridge` is the authority on that: a fragment the
/// planner can compile ahead of time resolves its names through ordinary lowering, where the
/// reachability fixpoint already sees them; one that still needs the bridge resolves them at
/// runtime and can reach anything, including through an `include`.
pub(crate) use runtime_features::eval_literal_call_requires_bridge;
mod declaration_metadata;
mod runtime_bindings;
mod function_declarations;
mod class_methods;
mod spl_discovery;
mod spl_metadata;
mod spl_support;
mod spl_lowering;

use metadata::*;
use runtime_features::*;
use eval_aot::*;
use declaration_metadata::*;
use function_declarations::*;
use class_methods::*;
use spl_discovery::*;
use spl_metadata::*;
use spl_support::*;
use spl_lowering::*;

pub(super) use eval_aot::all_lowered_functions;
pub(super) use runtime_features::include_lowered_runtime_features;
pub(super) use spl_discovery::{
    class_data_name, dynamic_object_new_metadata_names, php_method_key, string_data_name,
};
pub(super) use spl_lowering::class_method_already_lowered;

/// Lowers an optimized typed AST program into a validated EIR module.
///
/// `web` mirrors the CLI `--web` flag (the same source
/// `codegen_ir::block_emit::emit_module` receives) and is stored on the
/// returned module so every lowering entry point in `function.rs` can gate
/// request-superglobal type seeding on it; see `Module::web`.
pub(crate) fn lower(
    program: &Program,
    check_result: &CheckResult,
    target: Target,
    source_path: Option<&Path>,
    web: bool,
    source_catalog: Option<crate::ir::SourceCatalog>,
) -> Result<Module, LoweringError> {
    let _dynamic_function_resolution =
        super::context::enable_runtime_dynamic_function_resolution(body_contains_eval_call(program));
    let mut module = match source_catalog {
        Some(catalog) => Module::with_source_catalog(target, catalog),
        None => Module::new(target),
    };
    module.source_path = source_path.map(canonical_source_path);
    module.web = web;
    module.runtime_bound_functions = std::sync::Arc::new(
        runtime_bindings::collect_runtime_bound_functions(program),
    );
    module.ref_param_escaping_callees = check_result.ref_param_escaping_callees.clone();
    module.runtime_define_names = check_result.runtime_define_names.iter().cloned().collect();
    let _runtime_define_names =
        super::context::set_runtime_define_names(check_result.runtime_define_names.clone());
    let _undefined_constant_reads = crate::types::predefined_constants::set_undefined_constant_reads(
        check_result.undefined_constant_reads.clone(),
    );
    let _ref_param_escaping_callees = super::context::set_ref_param_escaping_callees(
        check_result.ref_param_escaping_callees.clone(),
    );
    let constants = crate::codegen::collect_constants(program, target);
    module.global_constants = constants.clone();
    let fiber_return_sigs = crate::ir_lower::fibers::collect_fiber_return_sigs(program);
    populate_metadata(&mut module, program, check_result);
    lower_function_declarations(
        program,
        &mut module,
        check_result,
        &constants,
        &fiber_return_sigs,
    );
    lower_class_like_methods(
        program,
        &mut module,
        check_result,
        &constants,
        &fiber_return_sigs,
    );
    lower_property_init_thunks(&mut module, check_result, &constants, &fiber_return_sigs);
    function::lower_main(
        program,
        &mut module,
        check_result,
        &constants,
        &fiber_return_sigs,
    );
    lower_literal_eval_aot_functions(&mut module, check_result, &constants, &fiber_return_sigs);
    lower_dynamic_constructor_thunks(&mut module, check_result, &constants, &fiber_return_sigs);
    include_lowered_runtime_features(&mut module);
    super::reflection::lower_referenced_builtin_methods(
        &mut module,
        check_result,
        &constants,
        &fiber_return_sigs,
    );
    lower_referenced_builtin_spl_methods(&mut module, check_result, &constants, &fiber_return_sigs);
    builtin_datetime::lower_referenced_builtin_datetime_methods(
        &mut module,
        check_result,
        &constants,
        &fiber_return_sigs,
    );
    include_lowered_runtime_features(&mut module);
    super::effect_refinement::refine_module(&mut module);
    if COLLECT_INVALID_BODIES.load(std::sync::atomic::Ordering::Relaxed) {
        let failures = invalid_bodies(&module);
        if let Some((_, first)) = failures.first() {
            let first = first.clone();
            return Err(LoweringError::InvalidBodies {
                first,
                failures: failures
                    .into_iter()
                    .map(|(name, error)| (name, format!("{error:?}")))
                    .collect(),
                module: Box::new(module),
            });
        }
    }
    validate_lowered_module(&module)?;
    Ok(module)
}

/// Whether validation collects every failing body instead of stopping at the first; set through
/// `crate::ir_lower::collect_invalid_bodies` by a build that can hand class files to the runtime.
pub(super) static COLLECT_INVALID_BODIES: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Returns every body of `module` that fails EIR validation, with its first validator error.
fn invalid_bodies(module: &Module) -> Vec<(String, ValidationError)> {
    module
        .functions
        .iter()
        .chain(module.class_methods.iter())
        .chain(module.closures.iter())
        .chain(module.fiber_wrappers.iter())
        .chain(module.callback_wrappers.iter())
        .chain(module.extern_callback_trampolines.iter())
        .chain(module.runtime_callable_invokers.iter())
        .filter_map(|function| {
            validate_function(function)
                .err()
                .map(|error| (function.name.clone(), error))
        })
        .collect()
}

/// Validates the lowered module, reporting every malformed body when backend inventory is active.
///
/// Ordinary builds preserve fail-fast validation. The inventory mode names every failing body and
/// its first validator error before returning the first error, extending the same scan-all contract
/// used by backend lowering to the EIR boundary that precedes it.
fn validate_lowered_module(module: &Module) -> Result<(), ValidationError> {
    if std::env::var("ELEPHC_BACKEND_INVENTORY").as_deref() != Ok("1") {
        return validate_module(module);
    }

    let mut first_error = None;
    for function in module
        .functions
        .iter()
        .chain(module.class_methods.iter())
        .chain(module.closures.iter())
        .chain(module.fiber_wrappers.iter())
        .chain(module.callback_wrappers.iter())
        .chain(module.extern_callback_trampolines.iter())
        .chain(module.runtime_callable_invokers.iter())
    {
        if let Err(error) = validate_function(function) {
            eprintln!("EIR validation failure in {}: {:?}", function.name, error);
            if let Some(context) = validation_error_context(function, &error) {
                eprintln!("  {context}");
            }
            // The whole refused body, for the refusals the one-line context cannot explain:
            // a dominance violation names two ids and says nothing about how the blocks relate.
            if std::env::var("ELEPHC_EIR_DUMP_INVALID").as_deref() == Ok("1") {
                eprintln!("{}", crate::ir::print_function(function));
            }
            first_error.get_or_insert(error);
        }
    }
    match first_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

/// Formats the producer and consumer metadata relevant to one validation refusal.
fn validation_error_context(function: &Function, error: &ValidationError) -> Option<String> {
    match error {
        ValidationError::ValueDefMismatch(value)
        | ValidationError::VoidValueUsed(value)
        | ValidationError::ResultTypeMismatch(value)
        | ValidationError::PhpTypeMismatch(value)
        | ValidationError::OwnershipTypeMismatch(value) => {
            Some(validation_value_context(function, *value))
        }
        ValidationError::OperandTypeMismatch { inst, operand, .. } => Some(format!(
            "consumer={:?}; {}",
            function.instruction(*inst),
            validation_value_context(function, *operand)
        )),
        _ => None,
    }
}

/// Formats one SSA value together with the instruction that produced it when available.
fn validation_value_context(function: &Function, value_id: ValueId) -> String {
    let value = function.value(value_id);
    let producer = value.and_then(|value| match value.def {
        ValueDef::Instruction { inst, .. } => validation_instruction(function, inst),
        ValueDef::BlockParam { .. } => None,
    });
    format!("value={value_id:?} metadata={value:?}; producer={producer:?}")
}

/// Returns an instruction by id for compact validation diagnostics.
fn validation_instruction(function: &Function, inst: InstId) -> Option<&crate::ir::Instruction> {
    function.instruction(inst)
}
