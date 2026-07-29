//! Purpose:
//! Audits a complete EIR module for wasm32-wasi capability and, after the
//! aggregate static gate succeeds, returns its exact lowered plan.
//!
//! Called from:
//! - `crate::codegen_wasm::generate` as the sole capability-and-planning gate.
//!
//! Key details:
//! - Every EIR function collection is inspected. Collections the backend cannot
//!   yet emit are rejected explicitly instead of being silently omitted.
//! - Opcode and terminator classification is exhaustive, so a new enum variant
//!   cannot compile until its WASM status is decided.
//! - Diagnostics name the collection, function, block, and instruction.
//! - A successful static audit delegates once to `plan::plan_module`; no WAT
//!   lowering is repeated after the returned plan is accepted.

use super::calls::{classify_by_ref_source, resolve_direct_call, ByRefSource};
use super::plan::{self, LoweredWasmPlan};
use super::transfer;
use super::WasmError;
use crate::codegen::Emit;
use crate::ir::{
    Function, Immediate, Instruction, IrHeapKind, IrType, Module, Op, RuntimeCallTarget,
    Ownership, RuntimeFnId, Terminator, UnaryStringRuntime, ValueDef, ValueId,
};
use crate::types::PhpType;
use std::collections::HashSet;

/// Compile-time provenance of ref-bound local slots within one EIR function.
///
/// `owned` means the slot points at a promotable kind-7 heap cell. `borrowed`
/// means the slot can point at caller-owned or interior array storage. A slot in
/// both sets is conservatively non-escapable.
#[derive(Default)]
struct RefCellProvenance {
    owned: HashSet<u32>,
    borrowed: HashSet<u32>,
}

/// Audits every function collection and returns one aggregate diagnostic.
fn audit_module(module: &Module) -> Result<(), WasmError> {
    let mut issues = Vec::new();
    scan_functions(module, "functions", &module.functions, true, &mut issues);
    scan_functions(
        module,
        "class_methods",
        &module.class_methods,
        true,
        &mut issues,
    );
    scan_functions(module, "closures", &module.closures, true, &mut issues);
    scan_functions(
        module,
        "fiber_wrappers",
        &module.fiber_wrappers,
        false,
        &mut issues,
    );
    scan_functions(
        module,
        "callback_wrappers",
        &module.callback_wrappers,
        false,
        &mut issues,
    );
    scan_functions(
        module,
        "extern_callback_trampolines",
        &module.extern_callback_trampolines,
        false,
        &mut issues,
    );
    scan_functions(
        module,
        "runtime_callable_invokers",
        &module.runtime_callable_invokers,
        false,
        &mut issues,
    );
    if issues.is_empty() {
        Ok(())
    } else {
        Err(WasmError::Unsupported(format!(
            "WASM capability audit found {} issue(s):\n{}",
            issues.len(),
            issues
                .iter()
                .map(|issue| format!("  - {issue}"))
                .collect::<Vec<_>>()
                .join("\n")
        )))
    }
}

/// Audits and exactly lowers a module into the private plan consumed by generation.
///
/// Static capability issues retain their stable aggregate diagnostic and stop
/// before lowering. Once this function returns a plan, every fallible lowering
/// and identifier-consistency check has already succeeded; artifact publication
/// still owns WAT assembly and binary validation.
pub(super) fn validate_module(
    module: &Module,
    emit: Emit,
) -> Result<LoweredWasmPlan, WasmError> {
    audit_module(module)?;
    plan::plan_module(module, emit)
}

/// Scans one module function collection and records collection-level omission.
fn scan_functions(
    module: &Module,
    collection: &str,
    functions: &[Function],
    emitted: bool,
    issues: &mut Vec<String>,
) {
    for function in functions {
        if !emitted {
            issues.push(format!(
                "{collection}::{}: function collection is not emitted by wasm32-wasi",
                function.name
            ));
        }
        scan_function(module, collection, function, issues);
    }
}

/// Scans types, instructions, runtime targets, and terminators in one function.
fn scan_function(
    module: &Module,
    collection: &str,
    function: &Function,
    issues: &mut Vec<String>,
) {
    let ref_cell_provenance = collect_ref_cell_provenance(function);
    if function.flags.is_main
        && (function.return_type != IrType::Void
            || function.return_php_type.codegen_repr() != PhpType::Void)
    {
        issues.push(format!(
            "{collection}::{}: main must declare a void return, got {:?}/{:?}",
            function.name,
            function.return_type,
            function.return_php_type.codegen_repr()
        ));
    }
    check_type(
        function.return_type,
        &format!("{collection}::{} return", function.name),
        issues,
    );
    for (index, param) in function.params.iter().enumerate() {
        check_type(
            param.ir_type,
            &format!("{collection}::{} param#{index}", function.name),
            issues,
        );
    }
    for local in &function.locals {
        check_type(
            local.ir_type,
            &format!(
                "{collection}::{} local#{}",
                function.name,
                local.id.as_raw()
            ),
            issues,
        );
    }
    for block in &function.blocks {
        for (index, value_id) in block.params.iter().enumerate() {
            match function.value(*value_id) {
                Some(value) => check_type(
                    value.ir_type,
                    &format!(
                        "{collection}::{} block#{} param#{index}",
                        function.name,
                        block.id.as_raw()
                    ),
                    issues,
                ),
                None => issues.push(format!(
                    "{collection}::{} block#{} param#{index}: missing value {:?}",
                    function.name,
                    block.id.as_raw(),
                    value_id
                )),
            }
        }
        for inst_id in &block.instructions {
            let Some(instruction) = function.instruction(*inst_id) else {
                issues.push(format!(
                    "{collection}::{} block#{} instruction#{:?}: missing instruction",
                    function.name,
                    block.id.as_raw(),
                    inst_id
                ));
                continue;
            };
            if !op_is_supported(instruction.op) {
                issues.push(format!(
                    "{collection}::{} block#{} instruction#{}: unsupported op {}",
                    function.name,
                    block.id.as_raw(),
                    inst_id.as_raw(),
                    instruction.op.name()
                ));
            }
            if instruction.op == Op::LanguageConstructCall && !function.flags.is_main {
                issues.push(format!(
                    "{collection}::{} block#{} instruction#{}: exit/die outside main cannot unwind caller-owned WASM frames",
                    function.name,
                    block.id.as_raw(),
                    inst_id.as_raw()
                ));
            }
            check_instruction_shape(
                module,
                collection,
                function,
                block.id.as_raw(),
                inst_id.as_raw(),
                instruction,
                &ref_cell_provenance,
                issues,
            );
            check_type(
                instruction.result_type,
                &format!(
                    "{collection}::{} block#{} instruction#{} result",
                    function.name,
                    block.id.as_raw(),
                    inst_id.as_raw()
                ),
                issues,
            );
            if instruction.op == Op::RuntimeCall {
                check_runtime_call(
                    module,
                    collection,
                    function,
                    block.id.as_raw(),
                    inst_id.as_raw(),
                    instruction,
                    issues,
                );
            }
        }
        match block.terminator.as_ref() {
            Some(terminator) if !terminator_is_supported(terminator) => issues.push(format!(
                "{collection}::{} block#{}: unsupported terminator {}",
                function.name,
                block.id.as_raw(),
                terminator_name(terminator)
            )),
            None => issues.push(format!(
                "{collection}::{} block#{}: missing terminator",
                function.name,
                block.id.as_raw()
            )),
            Some(Terminator::Return { value: Some(_) }) if function.flags.is_main => {
                issues.push(format!(
                    "{collection}::{} block#{}: main return value would be discarded before proc_exit(0)",
                    function.name,
                    block.id.as_raw()
                ));
            }
            _ => {}
        }
    }
}

/// Records an exact lowerer-shape defect for the audited P0 instruction subset.
///
/// Other admitted opcodes still rely on their late lowerer diagnostics until
/// their operand, immediate, result, and metadata contracts are audited here.
fn check_instruction_shape(
    module: &Module,
    collection: &str,
    function: &Function,
    block: u32,
    instruction: u32,
    inst: &Instruction,
    ref_cell_provenance: &RefCellProvenance,
    issues: &mut Vec<String>,
) {
    let issue = match inst.op {
        Op::ICheckedAdd | Op::ICheckedSub | Op::ICheckedMul => {
            checked_int_binop_shape_issue(function, inst)
        }
        Op::Cast => cast_shape_issue(function, inst),
        Op::ArrayGet | Op::ArrayGetSilent => array_get_shape_issue(function, inst),
        Op::ArrayToHash => array_to_hash_shape_issue(function, inst),
        Op::Call => {
            direct_call_shape_issue(module, function, inst, ref_cell_provenance)
        }
        Op::MethodCall | Op::NullsafeMethodCall => {
            method_call_shape_issue(module, function, inst)
        }
        Op::StaticMethodCall => static_method_call_shape_issue(module, function, inst),
        Op::ClosureCall => closure_call_shape_issue(module, function, inst),
        Op::CallableDescriptorInvoke => {
            callable_descriptor_invoke_shape_issue(module, function, inst)
        }
        Op::ClosureNew => {
            closure_new_by_ref_capture_issue(module, function, inst, ref_cell_provenance)
        }
        Op::FirstClassCallableNew => {
            first_class_callable_new_shape_issue(module, inst)
        }
        _ => None,
    };
    if let Some(issue) = issue {
        issues.push(format!(
            "{collection}::{} block#{block} instruction#{instruction}: unsupported {} shape: {issue}",
            function.name,
            inst.op.name()
        ));
    }
}

/// Collects conservative owned/borrowed ref-binding provenance for one function.
///
/// Promoted cells are owned; by-ref parameters and foreach element bindings are
/// borrowed. Alias edges propagate both classifications to a fixed point. Ambiguous
/// control-flow bindings therefore carry `borrowed` and cannot escape in a closure.
fn collect_ref_cell_provenance(function: &Function) -> RefCellProvenance {
    let mut provenance = RefCellProvenance::default();
    let mut aliases = Vec::new();
    for (index, param) in function.params.iter().enumerate() {
        if param.by_ref {
            provenance.borrowed.insert(index as u32);
        }
    }
    for inst in &function.instructions {
        match (inst.op, &inst.immediate) {
            (Op::PromoteLocalRefCell, Some(Immediate::LocalSlotPair { first, second })) => {
                provenance.owned.insert(first.as_raw());
                provenance.owned.insert(second.as_raw());
            }
            (Op::AliasLocalRefCell, Some(Immediate::LocalSlotPair { first, second })) => {
                aliases.push((first.as_raw(), second.as_raw()));
            }
            (Op::IterCurrentValueRef, Some(Immediate::LocalSlot(slot))) => {
                provenance.borrowed.insert(slot.as_raw());
            }
            _ => {}
        }
    }
    loop {
        let mut changed = false;
        for (target, source) in &aliases {
            if provenance.owned.contains(source) {
                changed |= provenance.owned.insert(*target);
            }
            if provenance.borrowed.contains(source) {
                changed |= provenance.borrowed.insert(*target);
            }
        }
        if !changed {
            break;
        }
    }
    provenance
}

/// Reports a non-escapable by-ref `ClosureNew` operand before WAT emission.
///
/// Fresh `LoadLocal` operands are promotable. `LoadRefCell` operands require an
/// unambiguous owned provenance; borrowed by-ref parameters, foreach element
/// addresses, and aliases derived from either are rejected.
fn closure_new_by_ref_capture_issue(
    module: &Module,
    function: &Function,
    inst: &Instruction,
    provenance: &RefCellProvenance,
) -> Option<String> {
    let Some(name) = data_string(module, inst) else {
        return Some("missing closure-name data immediate".to_string());
    };
    let closures: Vec<&Function> = module
        .closures
        .iter()
        .filter(|closure| closure.name == name)
        .collect();
    let closure = match closures.as_slice() {
        [closure] => *closure,
        [] => return Some(format!("closure body {name:?} is missing")),
        _ => {
            return Some(format!(
                "closure body {name:?} is ambiguous across {} definitions",
                closures.len()
            ))
        }
    };
    let capture_count = inst.operands.len();
    if capture_count != closure.flags.closure_capture_count {
        return Some(format!(
            "operand count {capture_count} does not match closure capture_count {}",
            closure.flags.closure_capture_count
        ));
    }
    let Some(visible_count) = closure.params.len().checked_sub(capture_count) else {
        return Some(format!(
            "capture count {capture_count} exceeds closure parameter count {}",
            closure.params.len()
        ));
    };
    if let Some(issue) = callable_wrapper_signature_issue(closure, visible_count) {
        return Some(issue);
    }
    if !callable_return_is_boxable(closure) {
        return Some(format!(
            "callable return {:?}/{:?} cannot be boxed by the WASM wrapper",
            closure.return_type,
            closure.return_php_type.codegen_repr()
        ));
    }
    for (operand, param) in inst
        .operands
        .iter()
        .zip(&closure.params[visible_count..])
    {
        if !param.by_ref {
            continue;
        }
        let Some(value) = function.value(*operand) else {
            return Some(format!(
                "by-ref capture {} operand {:?} is missing",
                param.name, operand
            ));
        };
        let ValueDef::Instruction { inst: source_id, .. } = value.def else {
            return Some(format!(
                "by-ref capture {} source is not a local load",
                param.name
            ));
        };
        let Some(source) = function.instruction(source_id) else {
            return Some(format!(
                "by-ref capture {} source instruction {:?} is missing",
                param.name, source_id
            ));
        };
        match (source.op, &source.immediate) {
            (Op::LoadLocal, Some(Immediate::LocalSlot(_))) => {}
            (Op::LoadRefCell, Some(Immediate::LocalSlot(slot)))
                if provenance.owned.contains(&slot.as_raw())
                    && !provenance.borrowed.contains(&slot.as_raw()) => {}
            (Op::LoadRefCell, Some(Immediate::LocalSlot(slot))) => {
                return Some(format!(
                    "by-ref capture {} cannot escape non-owned ref-bound local#{}",
                    param.name,
                    slot.as_raw()
                ));
            }
            _ => {
                return Some(format!(
                    "by-ref capture {} source must be LoadLocal or an owned LoadRefCell",
                    param.name
                ));
            }
        }
    }
    None
}

/// Validates one free-function first-class callable against PHP name folding,
/// collision rules, and the wrapper's supported parameter/result shapes.
fn first_class_callable_new_shape_issue(
    module: &Module,
    inst: &Instruction,
) -> Option<String> {
    let Some(name) = data_string(module, inst) else {
        return Some("missing callable-name Data immediate".to_string());
    };
    let key = crate::names::php_symbol_key(name.trim_start_matches('\\'));
    let matches: Vec<&Function> = module
        .functions
        .iter()
        .filter(|function| {
            !function.flags.is_main
                && crate::names::php_symbol_key(function.name.trim_start_matches('\\')) == key
        })
        .collect();
    let function = match matches.as_slice() {
        [function] => *function,
        [] => return Some(format!("free-function callable target {name:?} is missing")),
        _ => {
            return Some(format!(
                "free-function callable target {name:?} is ambiguous across {} definitions",
                matches.len()
            ))
        }
    };
    if let Some(issue) = callable_wrapper_signature_issue(function, function.params.len()) {
        return Some(issue);
    }
    if !callable_return_is_boxable(function) {
        return Some(format!(
            "callable return {:?}/{:?} cannot be boxed by the WASM wrapper",
            function.return_type,
            function.return_php_type.codegen_repr()
        ));
    }
    None
}

/// Validates the two-integer to owned-Mixed contract of checked arithmetic.
fn checked_int_binop_shape_issue(function: &Function, inst: &Instruction) -> Option<String> {
    let [lhs, rhs] = inst.operands.as_slice() else {
        return Some(format!(
            "expected two integer operands, got {}",
            inst.operands.len()
        ));
    };
    for (label, operand) in [("lhs", lhs), ("rhs", rhs)] {
        let Some(value) = function.value(*operand) else {
            return Some(format!("{label} operand is missing from the value table"));
        };
        if value.ir_type != IrType::I64 || value.php_type.codegen_repr() != PhpType::Int {
            return Some(format!(
                "{label} must be int/I64, got {:?}/{:?}",
                value.ir_type,
                value.php_type.codegen_repr()
            ));
        }
    }
    if inst.result.is_none()
        || inst.result_type != IrType::Heap(IrHeapKind::Mixed)
        || inst.result_php_type.codegen_repr() != PhpType::Mixed
    {
        return Some(format!(
            "checked arithmetic must materialize a Mixed cell, got {:?}/{:?}",
            inst.result_type,
            inst.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Validates the exact source/target pairs implemented by `lower_cast`.
fn cast_shape_issue(function: &Function, inst: &Instruction) -> Option<String> {
    let [operand] = inst.operands.as_slice() else {
        return Some(format!(
            "expected one source operand, got {}",
            inst.operands.len()
        ));
    };
    let Some(source) = function.value(*operand) else {
        return Some("cast source is missing from the value table".to_string());
    };
    let Some(Immediate::CastTarget(target)) = inst.immediate else {
        return Some("missing CastTarget immediate".to_string());
    };
    if inst.result.is_none() || target != inst.result_type {
        return Some(format!(
            "cast target {target:?} must equal the materialized result {:?}",
            inst.result_type
        ));
    }

    let source_php = source.php_type.codegen_repr();
    let result_php = inst.result_php_type.codegen_repr();
    let supported = match (source.ir_type, target) {
        (IrType::Heap(IrHeapKind::Mixed), IrType::I64) => {
            matches!(&source_php, PhpType::Mixed)
                && matches!(&result_php, PhpType::Int | PhpType::Bool)
        }
        (IrType::Heap(IrHeapKind::Mixed), IrType::F64) => {
            source_php == PhpType::Mixed && result_php == PhpType::Float
        }
        (IrType::Heap(IrHeapKind::Mixed), IrType::Str) => {
            source_php == PhpType::Mixed && result_php == PhpType::Str
        }
        (IrType::TaggedScalar, IrType::I64) => {
            source_php == PhpType::TaggedScalar
                && matches!(result_php, PhpType::Int | PhpType::Bool)
        }
        (IrType::TaggedScalar, IrType::F64) => {
            source_php == PhpType::TaggedScalar && result_php == PhpType::Float
        }
        (IrType::I64, IrType::I64) => {
            matches!(source_php, PhpType::Int | PhpType::Bool) && source_php == result_php
        }
        (IrType::F64, IrType::F64) => {
            source_php == PhpType::Float && result_php == PhpType::Float
        }
        (IrType::Str, IrType::Str) => {
            source_php == PhpType::Str && result_php == PhpType::Str
        }
        (IrType::I64, IrType::F64) => {
            source_php == PhpType::Int && result_php == PhpType::Float
        }
        (IrType::F64, IrType::I64) => {
            source_php == PhpType::Float && result_php == PhpType::Int
        }
        _ => false,
    };
    if !supported {
        return Some(format!(
            "unsupported conversion {:?}/{source_php:?} to {target:?}/{result_php:?}",
            source.ir_type
        ));
    }
    None
}

/// Validates null-capable indexed int/bool/string reads supported by `lower_array_get`.
///
/// The warning distinction between `ArrayGet` and `ArrayGetSilent` is a separate
/// runtime diagnostic gate; both opcodes must preserve the same value/null shape.
fn array_get_shape_issue(function: &Function, inst: &Instruction) -> Option<String> {
    let [array, index] = inst.operands.as_slice() else {
        return Some(format!(
            "expected an indexed array and integer index, got {} operands",
            inst.operands.len()
        ));
    };
    let Some(array_value) = function.value(*array) else {
        return Some("array operand is missing from the value table".to_string());
    };
    let element_type = match (
        array_value.ir_type,
        array_value.php_type.codegen_repr(),
    ) {
        (IrType::Heap(IrHeapKind::Array), PhpType::Array(element)) => {
            element.codegen_repr()
        }
        (ir_type, php_type) => {
            return Some(format!(
                "source must be an indexed array, got {ir_type:?}/{php_type:?}"
            ))
        }
    };
    let Some(index_value) = function.value(*index) else {
        return Some("index operand is missing from the value table".to_string());
    };
    if index_value.ir_type != IrType::I64
        || index_value.php_type.codegen_repr() != PhpType::Int
    {
        return Some(format!(
            "index must be int/I64, got {:?}/{:?}",
            index_value.ir_type,
            index_value.php_type.codegen_repr()
        ));
    }
    if inst.result.is_none() {
        return Some("array_get must materialize its result".to_string());
    }
    let result_php = inst.result_php_type.codegen_repr();
    let supported_result = match &element_type {
        PhpType::Int => {
            inst.result_type == IrType::TaggedScalar && result_php == PhpType::TaggedScalar
        }
        PhpType::Bool | PhpType::Str => {
            inst.result_type == IrType::Heap(IrHeapKind::Mixed) && result_php == PhpType::Mixed
        }
        _ => false,
    };
    if !supported_result {
        return Some(format!(
            "element {element_type:?} cannot lower into {:?}/{result_php:?}",
            inst.result_type
        ));
    }
    None
}

/// Validates the consuming indexed-array to associative-hash promotion contract.
///
/// The runtime consumes an owned/maybe-owned indexed source and returns a
/// release-tracked hash. Concrete element storage may be preserved exactly or
/// widened to Mixed, as required by array spread and contextual hash promotion;
/// the empty-literal `array<never>` placeholder may adopt its contextual value type.
fn array_to_hash_shape_issue(function: &Function, inst: &Instruction) -> Option<String> {
    let [array] = inst.operands.as_slice() else {
        return Some(format!(
            "expected one indexed-array operand, got {}",
            inst.operands.len()
        ));
    };
    if inst.immediate.is_some() {
        return Some("promotion must not carry an immediate".to_string());
    }
    let Some(source) = function.value(*array) else {
        return Some("array operand is missing from the value table".to_string());
    };
    let source_element = match (source.ir_type, source.php_type.codegen_repr()) {
        (IrType::Heap(IrHeapKind::Array), PhpType::Array(element)) => {
            element.codegen_repr()
        }
        (ir_type, php_type) => {
            return Some(format!(
                "source must be an indexed array, got {ir_type:?}/{php_type:?}"
            ))
        }
    };
    if !matches!(source.ownership, Ownership::Owned | Ownership::MaybeOwned) {
        return Some(format!(
            "consumed source must own a releasable reference, got {:?}",
            source.ownership
        ));
    }
    let result_value = match inst.result_php_type.codegen_repr() {
        PhpType::AssocArray { key, value } if key.codegen_repr() == PhpType::Int => {
            value.codegen_repr()
        }
        php_type => {
            return Some(format!(
                "result must be AssocArray<Int, T>, got {:?}/{php_type:?}",
                inst.result_type
            ))
        }
    };
    if inst.result.is_none() || inst.result_type != IrType::Heap(IrHeapKind::Hash) {
        return Some(format!(
            "result must materialize Heap(Hash), got {:?}",
            inst.result_type
        ));
    }
    if !matches!(
        inst.result_ownership,
        Ownership::Owned | Ownership::MaybeOwned
    ) {
        return Some(format!(
            "result must own a releasable hash reference, got {:?}",
            inst.result_ownership
        ));
    }
    if source_element != PhpType::Void
        && result_value != source_element
        && result_value != PhpType::Mixed
    {
        return Some(format!(
            "result value {result_value:?} must preserve {source_element:?} or widen to Mixed"
        ));
    }
    None
}

/// Validates one direct-call target, its lowered arity, storage transfers, and
/// by-reference source forms against the exact resolver used by emission.
fn direct_call_shape_issue(
    module: &Module,
    owner: &Function,
    inst: &Instruction,
    ref_cell_provenance: &RefCellProvenance,
) -> Option<String> {
    let target = match resolve_direct_call(module, inst) {
        Ok(target) => target,
        Err(error) => return Some(error.to_string()),
    };
    if target.function.params.iter().any(|param| param.variadic) {
        return Some(format!(
            "target {:?} has a variadic parameter outside the L1 direct-call contract",
            target.name
        ));
    }
    if target.function.params.len() != inst.operands.len() {
        return Some(format!(
            "target {:?} expects {} lowered operands, got {}",
            target.name,
            target.function.params.len(),
            inst.operands.len()
        ));
    }
    for (index, (operand, parameter)) in inst
        .operands
        .iter()
        .zip(&target.function.params)
        .enumerate()
    {
        let Some(value) = owner.value(*operand) else {
            return Some(format!("argument #{index} is missing from the value table"));
        };
        if parameter.by_ref {
            let source = classify_by_ref_source(owner, *operand);
            if let Some(issue) =
                by_ref_source_shape_issue(owner, value, parameter, source)
            {
                return Some(format!("by-reference argument #{index}: {issue}"));
            }
            match source {
                ByRefSource::NonLocal => {
                    return Some(format!(
                        "by-reference argument #{index} is not a supported local storage load"
                    ))
                }
                ByRefSource::AlreadyRefBound(slot)
                    if !ref_cell_provenance.owned.contains(&slot)
                        && !ref_cell_provenance.borrowed.contains(&slot) =>
                {
                    return Some(format!(
                        "by-reference argument #{index} reads unregistered ref-cell local#{slot}"
                    ))
                }
                ByRefSource::AlreadyRefBound(_) | ByRefSource::FreshLocal(_) => {}
            }
            if value.ir_type != parameter.ir_type
                || value.php_type.codegen_repr() != parameter.php_type.codegen_repr()
            {
                return Some(format!(
                    "by-reference argument #{index} storage {:?}/{:?} differs from parameter {:?}/{:?}",
                    value.ir_type,
                    value.php_type.codegen_repr(),
                    parameter.ir_type,
                    parameter.php_type.codegen_repr()
                ));
            }
        } else if let Err(error) = transfer::classify_transfer(
            value.ir_type,
            value.php_type.codegen_repr(),
            parameter.ir_type,
            parameter.php_type.codegen_repr(),
        ) {
            return Some(format!("argument #{index}: {error}"));
        }
    }
    if let Some(result) = inst.result {
        let Some(value) = owner.value(result) else {
            return Some("call result is missing from the value table".to_string());
        };
        if let Err(error) = transfer::classify_transfer(
            target.function.return_type,
            target.function.return_php_type.codegen_repr(),
            value.ir_type,
            value.php_type.codegen_repr(),
        ) {
            return Some(format!("result: {error}"));
        }
    }
    None
}

/// Validates the exact local/value/parameter metadata behind one by-reference
/// direct-call argument before emission takes or promotes its cell address.
fn by_ref_source_shape_issue(
    owner: &Function,
    value: &crate::ir::Value,
    parameter: &crate::ir::FunctionParam,
    source: ByRefSource,
) -> Option<String> {
    let slot_raw = match source {
        ByRefSource::AlreadyRefBound(slot) => slot,
        ByRefSource::FreshLocal(slot) => slot.as_raw(),
        ByRefSource::NonLocal => return None,
    };
    let Some(slot) = owner
        .locals
        .iter()
        .find(|slot| slot.id.as_raw() == slot_raw)
    else {
        return Some(format!("local#{slot_raw} is missing from the local table"));
    };
    let pair_is_valid = transfer::validate_storage_pair(slot.ir_type, &slot.php_type).is_ok()
        && transfer::validate_storage_pair(value.ir_type, &value.php_type).is_ok()
        && transfer::validate_storage_pair(parameter.ir_type, &parameter.php_type).is_ok();
    let exact_metadata = slot.ir_type == value.ir_type
        && value.ir_type == parameter.ir_type
        && slot.php_type == value.php_type
        && value.php_type == parameter.php_type;
    if !pair_is_valid || !exact_metadata {
        return Some(format!(
            "local#{slot_raw} metadata {:?}/{:?}, loaded value {:?}/{:?}, and parameter {:?}/{:?} must match exactly",
            slot.ir_type,
            slot.php_type,
            value.ir_type,
            value.php_type,
            parameter.ir_type,
            parameter.php_type
        ));
    }
    None
}

/// Validates the statically resolvable object and boxed Mixed/Union method paths.
///
/// A Mixed/Union receiver can still carry a non-object runtime tag or an object
/// outside the closed candidate set. Those dynamic cases route through the PHP
/// fatal helpers; this gate proves that every closed-world candidate shares the
/// argument and result contract needed before runtime selection.
fn method_call_shape_issue(
    module: &Module,
    function: &Function,
    inst: &Instruction,
) -> Option<String> {
    let Some(method_name) = data_string(module, inst) else {
        return Some("missing or invalid method-name Data immediate".to_string());
    };
    let Some((receiver, arguments)) = inst.operands.split_first() else {
        return Some("missing receiver operand".to_string());
    };
    let Some(receiver_value) = function.value(*receiver) else {
        return Some("receiver operand is missing from the value table".to_string());
    };
    let method_key = crate::names::php_symbol_key(method_name);
    match receiver_value.php_type.codegen_repr() {
        PhpType::Object(class_name) => {
            if receiver_value.ir_type != IrType::Heap(IrHeapKind::Object) {
                return Some(format!(
                    "object receiver must use Heap(Object), got {:?}",
                    receiver_value.ir_type
                ));
            }
            let Some(class_info) = module.class_infos.get(&class_name) else {
                return Some(format!("unknown receiver class {class_name}"));
            };
            let Some(signature) = class_info.methods.get(&method_key) else {
                return Some(format!("unknown method {class_name}::{method_name}"));
            };
            if let Some(issue) = method_signature_shape_issue(
                function,
                arguments,
                signature,
                method_name,
            ) {
                return Some(issue);
            }
            let implementation = class_info
                .method_impl_classes
                .get(&method_key)
                .map(String::as_str)
                .unwrap_or(class_name.as_str());
            let Some(body) = find_method_function(module, implementation, &method_key) else {
                return Some(format!(
                    "missing method body {implementation}::{method_name}"
                ));
            };
            if let Some(issue) = method_body_signature_shape_issue(
                body,
                signature,
                IrType::Heap(IrHeapKind::Object),
            ) {
                return Some(issue);
            }
            if let Some(issue) = method_body_argument_shape_issue(function, inst, body) {
                return Some(issue);
            }
            direct_method_result_shape_issue(inst, body, &signature.return_type)
        }
        PhpType::Mixed | PhpType::Union(_) => {
            if !matches!(
                receiver_value.ir_type,
                IrType::Heap(IrHeapKind::Mixed | IrHeapKind::Union)
            ) {
                return Some(format!(
                    "dynamic receiver must use Heap(Mixed/Union), got {:?}",
                    receiver_value.ir_type
                ));
            }
            let result_php = inst.result_php_type.codegen_repr();
            if transfer::validate_storage_pair(inst.result_type, &inst.result_php_type).is_err()
                && !(inst.result.is_none()
                    && inst.result_type == IrType::Void
                    && result_php == PhpType::Void)
            {
                return Some(format!(
                    "dynamic mixed/union method dispatch has invalid result storage {:?}/{result_php:?}",
                    inst.result_type
                ));
            }
            let boxed_result = inst.result.is_some()
                && inst.result_type == IrType::Heap(IrHeapKind::Mixed)
                && result_php == PhpType::Mixed;
            if inst.op == Op::NullsafeMethodCall && !boxed_result {
                return Some(format!(
                    "dynamic nullsafe dispatch requires a boxed Mixed result, got {:?}/{result_php:?}",
                    inst.result_type
                ));
            }

            let candidates =
                super::classes::mixed_method_candidates(module, &method_key, inst.operands.len());
            if candidates.is_empty() {
                return Some(format!(
                    "no closed-world candidate for dynamic method {method_name}"
                ));
            }
            for (_, class_name, implementation) in candidates {
                let Some(class_info) = module.class_infos.get(&class_name) else {
                    return Some(format!("missing candidate class {class_name}"));
                };
                let Some(signature) = class_info.methods.get(&method_key) else {
                    return Some(format!(
                        "missing candidate signature {class_name}::{method_name}"
                    ));
                };
                if let Some(issue) = method_signature_shape_issue(
                    function,
                    arguments,
                    signature,
                    method_name,
                ) {
                    return Some(format!("{class_name}: {issue}"));
                }
                let Some(body) = find_method_function(module, &implementation, &method_key) else {
                    return Some(format!(
                        "missing candidate body {implementation}::{method_name}"
                    ));
                };
                if let Some(issue) = method_body_signature_shape_issue(
                    body,
                    signature,
                    IrType::Heap(IrHeapKind::Object),
                ) {
                    return Some(format!("{class_name}: {issue}"));
                }
                if let Some(issue) = method_body_argument_shape_issue(function, inst, body) {
                    return Some(format!("{class_name}: {issue}"));
                }
                if boxed_result {
                    if !mixed_method_return_is_boxable(body.return_type, &body.return_php_type) {
                        return Some(format!(
                            "{class_name}::{method_name} return {:?}/{:?} cannot be boxed",
                            body.return_type,
                            body.return_php_type.codegen_repr()
                        ));
                    }
                } else if let Some(issue) =
                    direct_method_result_shape_issue(inst, body, &signature.return_type)
                {
                    return Some(format!("{class_name}: {issue}"));
                }
            }
            None
        }
        other => Some(format!(
            "receiver must be Object, Mixed, or Union, got {other:?}"
        )),
    }
}

/// Validates true-static and lexical `self::`/`parent::` method calls against
/// the exact hidden-parameter ABI emitted by `methods::lower_static_method_call`.
fn static_method_call_shape_issue(
    module: &Module,
    owner: &Function,
    inst: &Instruction,
) -> Option<String> {
    let Some(target) = data_string(module, inst) else {
        return Some("missing or invalid static-target Data immediate".to_string());
    };
    let Some((receiver_label, method_name)) = target.rsplit_once("::") else {
        return Some(format!("malformed static target {target:?}"));
    };
    if receiver_label == "static" {
        return Some(format!(
            "static::{method_name} late-bound dispatch is outside the L1 call contract"
        ));
    }
    let current_class = owner
        .name
        .rsplit_once("::")
        .map(|(class_name, _)| class_name.to_string());
    let receiver_class = match receiver_label {
        "self" => match current_class.clone() {
            Some(class_name) => class_name,
            None => return Some("self:: outside a method".to_string()),
        },
        "parent" => {
            let Some(class_name) = current_class.as_ref() else {
                return Some("parent:: outside a method".to_string());
            };
            let Some(parent) = module
                .class_infos
                .get(class_name)
                .and_then(|class| class.parent.clone())
            else {
                return Some(format!("class {class_name} has no parent"));
            };
            parent
        }
        named => named.to_string(),
    };
    let Some(class_info) = module.class_infos.get(&receiver_class) else {
        return Some(format!("unknown class {receiver_class}"));
    };
    let method_key = crate::names::php_symbol_key(method_name);
    let true_static = class_info.static_methods.contains_key(&method_key);
    let lexical_instance = !true_static
        && matches!(receiver_label, "self" | "parent")
        && owner.flags.is_method
        && !owner.flags.is_static
        && class_info.methods.contains_key(&method_key);
    let (signature, implementation, hidden_ir) = if true_static {
        let Some(signature) = class_info.static_methods.get(&method_key) else {
            return Some(format!(
                "missing static signature {receiver_class}::{method_name}"
            ));
        };
        let implementation = class_info
            .static_method_impl_classes
            .get(&method_key)
            .map(String::as_str)
            .unwrap_or(receiver_class.as_str());
        (signature, implementation, IrType::I64)
    } else if lexical_instance {
        let Some(signature) = class_info.methods.get(&method_key) else {
            return Some(format!(
                "missing instance signature {receiver_class}::{method_name}"
            ));
        };
        let implementation = class_info
            .method_impl_classes
            .get(&method_key)
            .map(String::as_str)
            .unwrap_or(receiver_class.as_str());
        (signature, implementation, IrType::Heap(IrHeapKind::Object))
    } else {
        return Some(format!(
            "target {target:?} is neither a true static method nor an applicable lexical instance method"
        ));
    };
    if let Some(issue) =
        method_signature_shape_issue(owner, &inst.operands, signature, method_name)
    {
        return Some(issue);
    }
    let Some(body) = find_method_function(module, implementation, &method_key) else {
        return Some(format!(
            "missing method body {implementation}::{method_name}"
        ));
    };
    if let Some(issue) = method_body_signature_shape_issue(body, signature, hidden_ir) {
        return Some(issue);
    }
    for (index, (operand, parameter)) in
        inst.operands.iter().zip(body.params.iter().skip(1)).enumerate()
    {
        let Some(value) = owner.value(*operand) else {
            return Some(format!("argument #{index} is missing from the value table"));
        };
        if value.ir_type != parameter.ir_type
            || value.php_type.codegen_repr() != parameter.php_type.codegen_repr()
        {
            return Some(format!(
                "argument #{index} storage {:?}/{:?} differs from {} parameter {:?}/{:?}",
                value.ir_type,
                value.php_type.codegen_repr(),
                body.name,
                parameter.ir_type,
                parameter.php_type.codegen_repr()
            ));
        }
    }
    direct_method_result_shape_issue(inst, body, &signature.return_type)
}

/// Validates user-argument arity, by-reference state, and PHP parameter types.
fn method_signature_shape_issue(
    owner: &Function,
    arguments: &[ValueId],
    signature: &crate::types::FunctionSig,
    method_name: &str,
) -> Option<String> {
    if signature.variadic.is_some() || signature.ref_params.iter().any(|by_ref| *by_ref) {
        return Some(format!(
            "{method_name} has a variadic or by-reference parameter"
        ));
    }
    if signature.params.len() != arguments.len() {
        return Some(format!(
            "{method_name} expects {} arguments, got {}",
            signature.params.len(),
            arguments.len()
        ));
    }
    for (index, (argument, (_, expected))) in
        arguments.iter().zip(&signature.params).enumerate()
    {
        let Some(value) = owner.value(*argument) else {
            return Some(format!("argument #{index} is missing from the value table"));
        };
        if value.php_type.codegen_repr() != expected.codegen_repr() {
            return Some(format!(
                "argument #{index} has PHP type {:?}, expected {:?}",
                value.php_type.codegen_repr(),
                expected.codegen_repr()
            ));
        }
    }
    None
}

/// Validates one concrete method body's hidden ABI parameter, user parameter
/// metadata, and return metadata against the checker-owned signature.
fn method_body_signature_shape_issue(
    body: &Function,
    signature: &crate::types::FunctionSig,
    hidden_ir: IrType,
) -> Option<String> {
    if body.params.len() != signature.params.len() + 1 {
        return Some(format!(
            "method body {} expects {} total parameters including its hidden receiver, signature requires {}",
            body.name,
            body.params.len(),
            signature.params.len() + 1
        ));
    }
    let Some(hidden) = body.params.first() else {
        return Some(format!("method body {} has no hidden parameter", body.name));
    };
    let hidden_php = hidden.php_type.codegen_repr();
    let hidden_php_matches = match hidden_ir {
        IrType::I64 => hidden_php == PhpType::Int,
        IrType::Heap(IrHeapKind::Object) => matches!(hidden_php, PhpType::Object(_)),
        _ => false,
    };
    if hidden.ir_type != hidden_ir
        || !hidden_php_matches
        || transfer::validate_storage_pair(hidden.ir_type, &hidden.php_type).is_err()
    {
        return Some(format!(
            "method body {} hidden parameter must be {:?} with matching PHP metadata, got {:?}/{hidden_php:?}",
            body.name, hidden_ir, hidden.ir_type
        ));
    }
    for (index, (parameter, (_, declared_php))) in body
        .params
        .iter()
        .skip(1)
        .zip(&signature.params)
        .enumerate()
    {
        let parameter_php = parameter.php_type.codegen_repr();
        let declared_php = declared_php.codegen_repr();
        if parameter_php != declared_php
            || transfer::validate_storage_pair(parameter.ir_type, &parameter.php_type).is_err()
        {
            return Some(format!(
                "method body {} parameter #{index} {:?}/{parameter_php:?} differs from signature {declared_php:?}",
                body.name, parameter.ir_type
            ));
        }
    }
    let body_return_php = body.return_php_type.codegen_repr();
    let declared_return_php = signature.return_type.codegen_repr();
    if body_return_php != declared_return_php
        || transfer::validate_storage_pair(body.return_type, &body.return_php_type).is_err()
    {
        return Some(format!(
            "method body {} return {:?}/{body_return_php:?} differs from signature {declared_return_php:?}",
            body.name, body.return_type
        ));
    }
    None
}

/// Validates argument storage against the concrete method body's WASM signature.
fn method_body_argument_shape_issue(
    owner: &Function,
    inst: &Instruction,
    body: &Function,
) -> Option<String> {
    if body.params.len() != inst.operands.len() {
        return Some(format!(
            "method body {} expects {} total operands, got {}",
            body.name,
            body.params.len(),
            inst.operands.len()
        ));
    }
    if body
        .params
        .first()
        .is_none_or(|parameter| parameter.ir_type != IrType::Heap(IrHeapKind::Object))
    {
        return Some(format!(
            "method body {} has no Heap(Object) receiver parameter",
            body.name
        ));
    }
    for (index, (operand, parameter)) in inst
        .operands
        .iter()
        .skip(1)
        .zip(body.params.iter().skip(1))
        .enumerate()
    {
        let Some(value) = owner.value(*operand) else {
            return Some(format!(
                "operand #{} is missing from the value table",
                index + 1
            ));
        };
        if value.ir_type != parameter.ir_type
            || value.php_type.codegen_repr() != parameter.php_type.codegen_repr()
        {
            return Some(format!(
                "operand #{} storage {:?}/{:?} differs from {} parameter {:?}/{:?}",
                index + 1,
                value.ir_type,
                value.php_type.codegen_repr(),
                body.name,
                parameter.ir_type,
                parameter.php_type.codegen_repr()
            ));
        }
    }
    None
}

/// Validates the direct object-call result against the concrete method body.
fn direct_method_result_shape_issue(
    inst: &Instruction,
    body: &Function,
    declared_php: &PhpType,
) -> Option<String> {
    let declared_php = declared_php.codegen_repr();
    let body_php = body.return_php_type.codegen_repr();
    if body_php != declared_php
        || transfer::validate_storage_pair(body.return_type, &body.return_php_type).is_err()
    {
        return Some(format!(
            "method body {} return {:?}/{body_php:?} differs from declared {declared_php:?}",
            body.name, body.return_type
        ));
    }
    let has_exact_result = match body.return_type {
        IrType::Void => {
            inst.result.is_none()
                && inst.result_type == IrType::Void
                && inst.result_php_type.codegen_repr() == PhpType::Void
        }
        return_type => {
            inst.result.is_some()
                && inst.result_type == return_type
                && inst.result_php_type.codegen_repr() == declared_php
        }
    };
    if !has_exact_result {
        return Some(format!(
            "result {:?}/{:?} differs from method body {:?}/{declared_php:?}",
            inst.result_type,
            inst.result_php_type.codegen_repr(),
            body.return_type
        ));
    }
    None
}

/// Finds a concrete class-method body using the lowerer's name-matching rules.
fn find_method_function<'a>(
    module: &'a Module,
    implementation: &str,
    method_key: &str,
) -> Option<&'a Function> {
    module
        .class_methods
        .iter()
        .find(|function| match function.name.rsplit_once("::") {
            Some((class_name, method_name)) => {
                class_name == implementation
                    && crate::names::php_symbol_key(method_name) == method_key
            }
            None => false,
        })
}

/// Returns whether dynamic dispatch can box the concrete method return.
fn mixed_method_return_is_boxable(ir_type: IrType, php_type: &PhpType) -> bool {
    transfer::classify_transfer(
        ir_type,
        php_type.codegen_repr(),
        IrType::Heap(IrHeapKind::Mixed),
        PhpType::Mixed,
    )
    .is_ok()
}

/// Records a storage type the current wasm32 ABI cannot materialize safely.
fn check_type(ir_type: IrType, context: &str, issues: &mut Vec<String>) {
    let supported = match ir_type {
        IrType::I64
        | IrType::F64
        | IrType::Str
        | IrType::TaggedScalar
        | IrType::Heap(IrHeapKind::Array)
        | IrType::Heap(IrHeapKind::Hash)
        | IrType::Heap(IrHeapKind::Object)
        | IrType::Heap(IrHeapKind::Mixed)
        | IrType::Heap(IrHeapKind::Iterable)
        | IrType::Heap(IrHeapKind::Union)
        | IrType::Void => true,
        IrType::Heap(IrHeapKind::Buffer) => false,
    };
    if !supported {
        issues.push(format!("{context}: unsupported storage type {ir_type:?}"));
    }
}

/// Validates the typed target carried by one `RuntimeCall`.
fn check_runtime_call(
    module: &Module,
    collection: &str,
    function: &Function,
    block: u32,
    instruction: u32,
    call: &Instruction,
    issues: &mut Vec<String>,
) {
    let context = format!(
        "{collection}::{} block#{block} instruction#{instruction}",
        function.name
    );
    match call.immediate.clone() {
        Some(Immediate::RuntimeCall(
            RuntimeCallTarget::Function(target)
            | RuntimeCallTarget::ProfiledFunction { target, .. },
        )) => {
            if !runtime_function_is_supported(target) {
                issues.push(format!(
                    "{context}: unsupported runtime function {}",
                    target.as_eir()
                ));
                return;
            }
            if let Some(issue) = runtime_function_shape_issue(module, function, call, target) {
                issues.push(format!(
                    "{context}: unsupported runtime function {} shape: {issue}",
                    target.as_eir()
                ));
            }
        }
        Some(Immediate::RuntimeCall(RuntimeCallTarget::ArrayFetchForWrite)) => {
            issues.push(format!(
                "{context}: unsupported runtime target array.fetch_for_write"
            ));
        }
        Some(Immediate::RuntimeCall(RuntimeCallTarget::UnaryString(target))) => {
            issues.push(format!(
                "{context}: unsupported unary string runtime {}",
                unary_string_name(target)
            ));
        }
        _ => issues.push(format!("{context}: missing typed runtime target")),
    }
}

/// Returns a diagnostic when an admitted runtime function does not match the
/// exact operand, callback, and result subset implemented by its lowerer.
fn runtime_function_shape_issue(
    module: &Module,
    function: &Function,
    call: &Instruction,
    target: RuntimeFnId,
) -> Option<String> {
    match target {
        RuntimeFnId::GetClass => get_class_shape_issue(function, call),
        RuntimeFnId::ArrayMap => array_map_shape_issue(module, function, call),
        RuntimeFnId::Usort => usort_shape_issue(module, function, call),
        RuntimeFnId::ArrayReduce => array_reduce_shape_issue(module, function, call),
        _ => Some("the runtime function has no audited WASM shape contract".to_string()),
    }
}

/// Validates the exact supported `get_class(object): string` form.
fn get_class_shape_issue(function: &Function, call: &Instruction) -> Option<String> {
    let [operand] = call.operands.as_slice() else {
        return Some(format!(
            "expected one object operand, got {}",
            call.operands.len()
        ));
    };
    let Some(value) = function.value(*operand) else {
        return Some("object operand is missing from the value table".to_string());
    };
    if value.ir_type != IrType::Heap(IrHeapKind::Object)
        || !matches!(value.php_type.codegen_repr(), PhpType::Object(_))
    {
        return Some(format!(
            "expected a statically object-typed Heap(Object) operand, got {:?}/{:?}",
            value.ir_type,
            value.php_type.codegen_repr()
        ));
    }
    if call.result.is_none()
        || call.result_type != IrType::Str
        || call.result_php_type.codegen_repr() != PhpType::Str
    {
        return Some(format!(
            "expected a string result, got {:?}/{:?}",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Validates the exact single-array `array_map` subset and its static callback.
fn array_map_shape_issue(
    module: &Module,
    function: &Function,
    call: &Instruction,
) -> Option<String> {
    let [callback, array] = call.operands.as_slice() else {
        return Some(format!(
            "expected (callable, indexed array), got {} operands",
            call.operands.len()
        ));
    };
    let Some(callback_value) = function.value(*callback) else {
        return Some("callback operand is missing from the value table".to_string());
    };
    if callback_value.php_type.codegen_repr() != PhpType::Callable {
        return Some(format!(
            "callback must be Callable, got {:?}",
            callback_value.php_type.codegen_repr()
        ));
    }
    let Some(array_value) = function.value(*array) else {
        return Some("array operand is missing from the value table".to_string());
    };
    let element_type = match (
        array_value.ir_type,
        array_value.php_type.codegen_repr(),
    ) {
        (IrType::Heap(IrHeapKind::Array), PhpType::Array(element)) => {
            element.codegen_repr()
        }
        (ir_type, php_type) => {
            return Some(format!(
                "source must be an indexed array, got {ir_type:?}/{php_type:?}"
            ))
        }
    };
    if !matches!(element_type, PhpType::Int | PhpType::Str) {
        return Some(format!(
            "source element type {element_type:?} is not represented exactly by the map runtime"
        ));
    }
    if call.result.is_none()
        || call.result_type != IrType::Heap(IrHeapKind::Array)
        || !matches!(
            call.result_php_type.codegen_repr(),
            PhpType::Array(_) | PhpType::AssocArray { .. }
        )
    {
        return Some(format!(
            "array_map must materialize an indexed-array result, got {:?}/{:?}",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    let Some((callback_function, visible_count)) =
        static_callable_contract(module, function, *callback)
    else {
        return Some(
            "callback must resolve statically to a direct closure or user-function callable"
                .to_string(),
        );
    };
    if let Some(issue) = callable_wrapper_issue(callback_function, visible_count, 1) {
        return Some(issue);
    }
    if let Some(param) = callback_function.params.first() {
        let param_type = param.php_type.codegen_repr();
        let compatible = match element_type {
            PhpType::Int => param_type == PhpType::Int,
            PhpType::Str => param_type == PhpType::Str,
            _ => false,
        };
        if !compatible {
            return Some(format!(
                "callback parameter {:?} cannot receive source element {:?} without PHP coercion",
                param_type, element_type
            ));
        }
    }
    if !callable_return_is_boxable(callback_function) {
        return Some(format!(
            "callback return {:?}/{:?} cannot be boxed by the WASM callable wrapper",
            callback_function.return_type,
            callback_function.return_php_type.codegen_repr()
        ));
    }
    None
}

/// Validates the exact `usort(array<int>, callable): bool` subset.
fn usort_shape_issue(
    module: &Module,
    function: &Function,
    call: &Instruction,
) -> Option<String> {
    let [array, callback] = call.operands.as_slice() else {
        return Some(format!(
            "expected (array<int>, callable), got {} operands",
            call.operands.len()
        ));
    };
    if !value_is_int_array(function, *array) {
        return Some("source must be an indexed array<int>".to_string());
    }
    if function
        .value(*callback)
        .is_none_or(|value| value.php_type.codegen_repr() != PhpType::Callable)
    {
        return Some("comparator must be Callable".to_string());
    }
    if call.result.is_some()
        && (call.result_type != IrType::I64
            || call.result_php_type.codegen_repr() != PhpType::Bool)
    {
        return Some(format!(
            "used usort result must be bool/I64, got {:?}/{:?}",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    let Some((callback_function, visible_count)) =
        static_callable_contract(module, function, *callback)
    else {
        return Some(
            "comparator must resolve statically to a direct closure or user-function callable"
                .to_string(),
        );
    };
    if let Some(issue) = callable_wrapper_issue(callback_function, visible_count, 2) {
        return Some(issue);
    }
    if callback_function.params[..visible_count]
        .iter()
        .any(|param| param.php_type.codegen_repr() != PhpType::Int)
    {
        return Some("comparator parameters must be int".to_string());
    }
    if callback_function.return_type != IrType::I64
        || callback_function.return_php_type.codegen_repr() != PhpType::Int
    {
        return Some(format!(
            "comparator must return int/I64, got {:?}/{:?}",
            callback_function.return_type,
            callback_function.return_php_type.codegen_repr()
        ));
    }
    None
}

/// Validates the exact integer-carry `array_reduce` subset.
fn array_reduce_shape_issue(
    module: &Module,
    function: &Function,
    call: &Instruction,
) -> Option<String> {
    let [array, callback, initial] = call.operands.as_slice() else {
        return Some(format!(
            "expected (array<int>, callable, int), got {} operands",
            call.operands.len()
        ));
    };
    if !value_is_int_array(function, *array) {
        return Some("source must be an indexed array<int>".to_string());
    }
    if function
        .value(*callback)
        .is_none_or(|value| value.php_type.codegen_repr() != PhpType::Callable)
    {
        return Some("callback must be Callable".to_string());
    }
    if function.value(*initial).is_none_or(|value| {
        value.ir_type != IrType::I64 || value.php_type.codegen_repr() != PhpType::Int
    }) {
        return Some("initial carry must be an int/I64 value".to_string());
    }
    if call.result.is_some() {
        let result_is_supported = match call.result_type {
            IrType::I64 => call.result_php_type.codegen_repr() == PhpType::Int,
            IrType::Heap(IrHeapKind::Mixed) => {
                call.result_php_type.codegen_repr() == PhpType::Mixed
            }
            IrType::Heap(IrHeapKind::Union) => {
                matches!(call.result_php_type.codegen_repr(), PhpType::Union(_))
            }
            _ => false,
        };
        if !result_is_supported {
            return Some(format!(
                "result must be int, Mixed, Union, or unused; got {:?}/{:?}",
                call.result_type,
                call.result_php_type.codegen_repr()
            ));
        }
    }
    let Some((callback_function, visible_count)) =
        static_callable_contract(module, function, *callback)
    else {
        return Some(
            "callback must resolve statically to a direct closure or user-function callable"
                .to_string(),
        );
    };
    if let Some(issue) = callable_wrapper_issue(callback_function, visible_count, 2) {
        return Some(issue);
    }
    if callback_function.params[..visible_count]
        .iter()
        .any(|param| param.php_type.codegen_repr() != PhpType::Int)
    {
        return Some("callback parameters must be int".to_string());
    }
    if callback_function.return_type != IrType::I64
        || callback_function.return_php_type.codegen_repr() != PhpType::Int
    {
        return Some(format!(
            "callback must return int/I64, got {:?}/{:?}",
            callback_function.return_type,
            callback_function.return_php_type.codegen_repr()
        ));
    }
    None
}

/// Returns whether a value is exactly an indexed `array<int>`.
fn value_is_int_array(function: &Function, value: ValueId) -> bool {
    function.value(value).is_some_and(|value| {
        value.ir_type == IrType::Heap(IrHeapKind::Array)
            && matches!(
                value.php_type.codegen_repr(),
                PhpType::Array(element) if element.codegen_repr() == PhpType::Int
            )
    })
}

/// Validates a direct `ClosureCall` argument buffer and its result unboxing.
///
/// Dynamic descriptors remain admitted because an escaping closure can be
/// loaded from a mutable local. When the descriptor producer is still visible
/// in SSA, its wrapper contract is additionally checked statically.
fn closure_call_shape_issue(
    module: &Module,
    owner: &Function,
    inst: &Instruction,
) -> Option<String> {
    let Some((callable, arguments)) = inst.operands.split_first() else {
        return Some("missing callable descriptor operand".to_string());
    };
    if !value_is_callable_descriptor(owner, *callable) {
        return Some("callable operand must be Callable/I64".to_string());
    }
    for (index, argument) in arguments.iter().enumerate() {
        let Some(value) = owner.value(*argument) else {
            return Some(format!("argument #{index} is missing from the value table"));
        };
        if !closure_argument_is_boxable(value.ir_type, &value.php_type) {
            return Some(format!(
                "argument #{index} cannot be boxed from {:?}/{:?}",
                value.ir_type,
                value.php_type.codegen_repr()
            ));
        }
    }
    if let Some(issue) = closure_result_shape_issue(owner, inst) {
        return Some(issue);
    }
    if let Some((target, visible_count)) = static_callable_contract(module, owner, *callable) {
        if let Some(issue) = callable_wrapper_issue(target, visible_count, arguments.len()) {
            return Some(issue);
        }
        if !callable_return_is_boxable(target) {
            return Some(format!(
                "callable return {:?}/{:?} cannot be boxed by the WASM wrapper",
                target.return_type,
                target.return_php_type.codegen_repr()
            ));
        }
    }
    None
}

/// Validates descriptor invocation through a pre-built positional
/// `array<mixed>` and the result-cell conversion performed by the lowerer.
fn callable_descriptor_invoke_shape_issue(
    module: &Module,
    owner: &Function,
    inst: &Instruction,
) -> Option<String> {
    let [callable, arguments] = inst.operands.as_slice() else {
        return Some(format!(
            "expected callable and positional array, got {} operands",
            inst.operands.len()
        ));
    };
    if !value_is_callable_descriptor(owner, *callable) {
        return Some("callable operand must be Callable/I64".to_string());
    }
    let Some(argument_value) = owner.value(*arguments) else {
        return Some("argument container is missing from the value table".to_string());
    };
    let is_mixed_array = argument_value.ir_type == IrType::Heap(IrHeapKind::Array)
        && matches!(
            argument_value.php_type.codegen_repr(),
            PhpType::Array(element) if element.codegen_repr() == PhpType::Mixed
        );
    if !is_mixed_array {
        return Some(format!(
            "argument container must be array<mixed>/Heap(Array), got {:?}/{:?}",
            argument_value.ir_type,
            argument_value.php_type.codegen_repr()
        ));
    }
    if let Some(issue) = closure_result_shape_issue(owner, inst) {
        return Some(issue);
    }
    if let Some((target, visible_count)) = static_callable_contract(module, owner, *callable) {
        if let Some(issue) = callable_wrapper_signature_issue(target, visible_count) {
            return Some(issue);
        }
        if !callable_return_is_boxable(target) {
            return Some(format!(
                "callable return {:?}/{:?} cannot be boxed by the WASM wrapper",
                target.return_type,
                target.return_php_type.codegen_repr()
            ));
        }
    }
    None
}

/// Returns whether one EIR value carries the descriptor representation consumed
/// by `__rt_closure_call`.
fn value_is_callable_descriptor(function: &Function, value: ValueId) -> bool {
    function.value(value).is_some_and(|value| {
        value.ir_type == IrType::I64 && value.php_type.codegen_repr() == PhpType::Callable
    })
}

/// Returns whether `objects::emit_box_value_into_mixed` implements this source.
fn closure_argument_is_boxable(ir_type: IrType, php_type: &PhpType) -> bool {
    if transfer::validate_storage_pair(ir_type, php_type).is_err() {
        return false;
    }
    let php_type = php_type.codegen_repr();
    match ir_type {
        IrType::I64 => matches!(
            php_type,
            PhpType::Int | PhpType::Bool | PhpType::Callable
        ),
        IrType::F64 => php_type == PhpType::Float,
        IrType::Str => php_type == PhpType::Str,
        IrType::Heap(IrHeapKind::Array) => matches!(php_type, PhpType::Array(_)),
        IrType::Heap(IrHeapKind::Hash) => matches!(php_type, PhpType::AssocArray { .. }),
        IrType::Heap(IrHeapKind::Object) => matches!(php_type, PhpType::Object(_)),
        IrType::Heap(
            IrHeapKind::Mixed | IrHeapKind::Iterable | IrHeapKind::Union | IrHeapKind::Buffer,
        )
        | IrType::TaggedScalar
        | IrType::Void => false,
    }
}

/// Returns the first unsupported closure result-cell destination shape.
fn closure_result_shape_issue(owner: &Function, inst: &Instruction) -> Option<String> {
    let Some(result) = inst.result else {
        return None;
    };
    let Some(value) = owner.value(result) else {
        return Some("result is missing from the value table".to_string());
    };
    if value.ir_type == IrType::I64
        && value.php_type.codegen_repr() == PhpType::Callable
        && transfer::validate_storage_pair(value.ir_type, &value.php_type).is_ok()
    {
        return None;
    }
    transfer::classify_transfer(
        IrType::Heap(IrHeapKind::Mixed),
        PhpType::Mixed,
        value.ir_type,
        value.php_type.codegen_repr(),
    )
    .err()
    .map(|error| format!("result cell cannot be stored: {error}"))
}

/// Resolves a direct closure/FCC descriptor through ownership-only SSA moves.
///
/// Dynamic callables, block parameters, and values loaded from mutable locals
/// remain rejected until callable descriptors carry a runtime signature.
fn static_callable_contract<'a>(
    module: &'a Module,
    owner: &'a Function,
    value: ValueId,
) -> Option<(&'a Function, usize)> {
    let mut current = value;
    for _ in 0..=owner.values.len() {
        let value = owner.value(current)?;
        let ValueDef::Instruction { inst, .. } = value.def else {
            return None;
        };
        let defining = owner.instruction(inst)?;
        match defining.op {
            Op::Move | Op::Borrow | Op::Acquire => {
                let [source] = defining.operands.as_slice() else {
                    return None;
                };
                current = *source;
            }
            Op::ClosureNew => {
                let name = data_string(module, defining)?;
                let function = module
                    .closures
                    .iter()
                    .find(|function| function.name == name)?;
                let visible = function
                    .params
                    .len()
                    .checked_sub(function.flags.closure_capture_count)?;
                return Some((function, visible));
            }
            Op::FirstClassCallableNew => {
                let name = data_string(module, defining)?;
                let key = crate::names::php_symbol_key(name.trim_start_matches('\\'));
                let function = module.functions.iter().find(|function| {
                    crate::names::php_symbol_key(function.name.trim_start_matches('\\')) == key
                })?;
                return Some((function, function.params.len()));
            }
            _ => return None,
        }
    }
    None
}

/// Resolves an ordinary or source-profiled data entry in the shared string pool.
fn data_string<'a>(module: &'a Module, instruction: &Instruction) -> Option<&'a str> {
    let data = match instruction.immediate {
        Some(Immediate::Data(data)) | Some(Immediate::ProfiledData { data, .. }) => data,
        _ => return None,
    };
    module
        .data
        .strings
        .get(data.as_raw() as usize)
        .map(String::as_str)
}

/// Returns the first wrapper-shape defect for a statically resolved callable.
fn callable_wrapper_issue(
    function: &Function,
    visible_count: usize,
    supplied_args: usize,
) -> Option<String> {
    if visible_count > supplied_args {
        return Some(format!(
            "callback declares {visible_count} visible parameters but the runtime supplies {supplied_args}"
        ));
    }
    callable_wrapper_signature_issue(function, visible_count)
}

/// Returns the first parameter-shape defect in a callable wrapper without
/// making a claim about the runtime argument-array length.
fn callable_wrapper_signature_issue(
    function: &Function,
    visible_count: usize,
) -> Option<String> {
    for param in &function.params[..visible_count] {
        if param.by_ref || param.variadic {
            return Some(format!(
                "callback parameter {} is by-reference or variadic",
                param.name
            ));
        }
        if !callable_param_is_unboxable(param) {
            return Some(format!(
                "callback parameter {} has unsupported storage {:?}/{:?}",
                param.name,
                param.ir_type,
                param.php_type.codegen_repr()
            ));
        }
    }
    None
}

/// Returns whether the callable wrapper can materialize one argument type.
fn callable_param_is_unboxable(param: &crate::ir::FunctionParam) -> bool {
    if transfer::validate_storage_pair(param.ir_type, &param.php_type).is_err() {
        return false;
    }
    let php_type = param.php_type.codegen_repr();
    match param.ir_type {
        IrType::I64 => matches!(
            php_type,
            PhpType::Int | PhpType::Bool | PhpType::Callable
        ),
        IrType::F64 => php_type == PhpType::Float,
        IrType::Str => php_type == PhpType::Str,
        IrType::Heap(IrHeapKind::Array) => matches!(php_type, PhpType::Array(_)),
        IrType::Heap(IrHeapKind::Hash) => matches!(php_type, PhpType::AssocArray { .. }),
        IrType::Heap(IrHeapKind::Object) => matches!(php_type, PhpType::Object(_)),
        IrType::Heap(
            IrHeapKind::Mixed | IrHeapKind::Iterable | IrHeapKind::Union | IrHeapKind::Buffer,
        )
        | IrType::TaggedScalar
        | IrType::Void => false,
    }
}

/// Returns whether the callable wrapper can box one callback result.
fn callable_return_is_boxable(function: &Function) -> bool {
    transfer::classify_transfer(
        function.return_type,
        function.return_php_type.codegen_repr(),
        IrType::Heap(IrHeapKind::Mixed),
        PhpType::Mixed,
    )
    .is_ok()
}

/// Returns whether a typed runtime function is safe to admit through the WASM gate.
///
/// A partial lowering stays rejected when its accepted subset changes PHP-visible
/// behavior. This keeps the capability audit conservative until the full public
/// contract is implemented and differentially tested.
fn runtime_function_is_supported(target: RuntimeFnId) -> bool {
    match target {
        RuntimeFnId::GetClass
        | RuntimeFnId::ArrayMap
        | RuntimeFnId::Usort
        | RuntimeFnId::ArrayReduce => true,
        RuntimeFnId::ArrayFilter
        | RuntimeFnId::Uasort
        | RuntimeFnId::Uksort
        | RuntimeFnId::ArrayWalk
        | RuntimeFnId::ArrayAll
        | RuntimeFnId::ArrayAny
        | RuntimeFnId::ArrayChunk
        | RuntimeFnId::ArrayColumn
        | RuntimeFnId::ArrayCombine
        | RuntimeFnId::ArrayDiff
        | RuntimeFnId::ArrayDiffAssoc
        | RuntimeFnId::ArrayDiffKey
        | RuntimeFnId::ArrayFill
        | RuntimeFnId::ArrayFillKeys
        | RuntimeFnId::ArrayFind
        | RuntimeFnId::ArrayFlip
        | RuntimeFnId::ArrayIntersect
        | RuntimeFnId::ArrayIntersectAssoc
        | RuntimeFnId::ArrayIntersectKey
        | RuntimeFnId::ArrayIsList
        | RuntimeFnId::ArrayKeyExists
        | RuntimeFnId::ArrayKeyFirst
        | RuntimeFnId::ArrayKeyLast
        | RuntimeFnId::ArrayKeys
        | RuntimeFnId::ArrayMerge
        | RuntimeFnId::ArrayMergeRecursive
        | RuntimeFnId::ArrayMultisort
        | RuntimeFnId::ArrayPad
        | RuntimeFnId::ArrayPop
        | RuntimeFnId::ArrayProduct
        | RuntimeFnId::ArrayPush
        | RuntimeFnId::ArrayRand
        | RuntimeFnId::ArrayReplace
        | RuntimeFnId::ArrayReplaceRecursive
        | RuntimeFnId::ArrayReverse
        | RuntimeFnId::ArraySearch
        | RuntimeFnId::ArrayShift
        | RuntimeFnId::ArraySlice
        | RuntimeFnId::ArraySplice
        | RuntimeFnId::ArraySum
        | RuntimeFnId::ArrayUdiff
        | RuntimeFnId::ArrayUintersect
        | RuntimeFnId::ArrayUnique
        | RuntimeFnId::ArrayUnshift
        | RuntimeFnId::ArrayValues
        | RuntimeFnId::ArrayWalkRecursive
        | RuntimeFnId::Arsort
        | RuntimeFnId::Asort
        | RuntimeFnId::Count
        | RuntimeFnId::InArray
        | RuntimeFnId::Krsort
        | RuntimeFnId::Ksort
        | RuntimeFnId::Natcasesort
        | RuntimeFnId::Natsort
        | RuntimeFnId::Range
        | RuntimeFnId::Rsort
        | RuntimeFnId::Shuffle
        | RuntimeFnId::Sort
        | RuntimeFnId::CallUserFunc
        | RuntimeFnId::CallUserFuncArray
        | RuntimeFnId::ClassAlias
        | RuntimeFnId::ClassExists
        | RuntimeFnId::ClassImplements
        | RuntimeFnId::ClassParents
        | RuntimeFnId::ClassUses
        | RuntimeFnId::EnumExists
        | RuntimeFnId::FunctionExists
        | RuntimeFnId::GetDeclaredClasses
        | RuntimeFnId::GetDeclaredInterfaces
        | RuntimeFnId::GetDeclaredTraits
        | RuntimeFnId::GetLoadedExtensions
        | RuntimeFnId::GetParentClass
        | RuntimeFnId::InterfaceExists
        | RuntimeFnId::IsA
        | RuntimeFnId::IsSubclassOf
        | RuntimeFnId::MethodExists
        | RuntimeFnId::PregReplaceCallback
        | RuntimeFnId::PropertyExists
        | RuntimeFnId::TraitExists
        | RuntimeFnId::ElephcPharBzip2Archive
        | RuntimeFnId::ElephcPharDecompressArchive
        | RuntimeFnId::ElephcPharGetFileMetadata
        | RuntimeFnId::ElephcPharGetMetadata
        | RuntimeFnId::ElephcPharGetSignatureHash
        | RuntimeFnId::ElephcPharGetSignatureType
        | RuntimeFnId::ElephcPharGetStub
        | RuntimeFnId::ElephcPharGzipArchive
        | RuntimeFnId::ElephcPharListEntries
        | RuntimeFnId::ElephcPharSetCompression
        | RuntimeFnId::ElephcPharSetFileMetadata
        | RuntimeFnId::ElephcPharSetMetadata
        | RuntimeFnId::ElephcPharSetStub
        | RuntimeFnId::ElephcPharSetZipPassword
        | RuntimeFnId::ElephcPharSignHash
        | RuntimeFnId::ElephcPharSignOpenssl
        | RuntimeFnId::Basename
        | RuntimeFnId::Chdir
        | RuntimeFnId::Chgrp
        | RuntimeFnId::Chmod
        | RuntimeFnId::Chown
        | RuntimeFnId::Clearstatcache
        | RuntimeFnId::Closedir
        | RuntimeFnId::Copy
        | RuntimeFnId::Dirname
        | RuntimeFnId::DiskFreeSpace
        | RuntimeFnId::DiskTotalSpace
        | RuntimeFnId::Fclose
        | RuntimeFnId::Fdatasync
        | RuntimeFnId::Feof
        | RuntimeFnId::Fflush
        | RuntimeFnId::Fgetc
        | RuntimeFnId::Fgetcsv
        | RuntimeFnId::Fgets
        | RuntimeFnId::File
        | RuntimeFnId::FileExists
        | RuntimeFnId::FileGetContents
        | RuntimeFnId::FilePutContents
        | RuntimeFnId::Fileatime
        | RuntimeFnId::Filectime
        | RuntimeFnId::Filegroup
        | RuntimeFnId::Fileinode
        | RuntimeFnId::Filemtime
        | RuntimeFnId::Fileowner
        | RuntimeFnId::Fileperms
        | RuntimeFnId::Filesize
        | RuntimeFnId::Filetype
        | RuntimeFnId::Flock
        | RuntimeFnId::Fnmatch
        | RuntimeFnId::Fopen
        | RuntimeFnId::Fpassthru
        | RuntimeFnId::Fprintf
        | RuntimeFnId::Fputcsv
        | RuntimeFnId::Fread
        | RuntimeFnId::Fscanf
        | RuntimeFnId::Fseek
        | RuntimeFnId::Fsockopen
        | RuntimeFnId::Fstat
        | RuntimeFnId::Fsync
        | RuntimeFnId::Ftell
        | RuntimeFnId::Ftruncate
        | RuntimeFnId::Fwrite
        | RuntimeFnId::Getcwd
        | RuntimeFnId::Gethostbyaddr
        | RuntimeFnId::Gethostbyname
        | RuntimeFnId::Gethostname
        | RuntimeFnId::Getprotobyname
        | RuntimeFnId::Getprotobynumber
        | RuntimeFnId::Getservbyname
        | RuntimeFnId::Getservbyport
        | RuntimeFnId::Glob
        | RuntimeFnId::HashFile
        | RuntimeFnId::IsDir
        | RuntimeFnId::IsExecutable
        | RuntimeFnId::IsFile
        | RuntimeFnId::IsLink
        | RuntimeFnId::IsReadable
        | RuntimeFnId::IsWritable
        | RuntimeFnId::IsWriteable
        | RuntimeFnId::Lchgrp
        | RuntimeFnId::Lchown
        | RuntimeFnId::Link
        | RuntimeFnId::Linkinfo
        | RuntimeFnId::Lstat
        | RuntimeFnId::Mkdir
        | RuntimeFnId::ObClean
        | RuntimeFnId::ObEndClean
        | RuntimeFnId::ObEndFlush
        | RuntimeFnId::ObFlush
        | RuntimeFnId::ObGetClean
        | RuntimeFnId::ObGetContents
        | RuntimeFnId::ObGetFlush
        | RuntimeFnId::ObGetLength
        | RuntimeFnId::ObGetLevel
        | RuntimeFnId::ObGetStatus
        | RuntimeFnId::ObImplicitFlush
        | RuntimeFnId::ObListHandlers
        | RuntimeFnId::ObStart
        | RuntimeFnId::Opendir
        | RuntimeFnId::Pathinfo
        | RuntimeFnId::Pclose
        | RuntimeFnId::Pfsockopen
        | RuntimeFnId::Popen
        | RuntimeFnId::PrintR
        | RuntimeFnId::Readdir
        | RuntimeFnId::Readfile
        | RuntimeFnId::Readline
        | RuntimeFnId::Readlink
        | RuntimeFnId::Realpath
        | RuntimeFnId::RealpathCacheGet
        | RuntimeFnId::RealpathCacheSize
        | RuntimeFnId::Rename
        | RuntimeFnId::Rewind
        | RuntimeFnId::Rewinddir
        | RuntimeFnId::Rmdir
        | RuntimeFnId::Scandir
        | RuntimeFnId::Stat
        | RuntimeFnId::StreamBucketAppend
        | RuntimeFnId::StreamBucketMakeWriteable
        | RuntimeFnId::StreamBucketNew
        | RuntimeFnId::StreamBucketPrepend
        | RuntimeFnId::StreamContextCreate
        | RuntimeFnId::StreamContextGetDefault
        | RuntimeFnId::StreamContextGetOptions
        | RuntimeFnId::StreamContextGetParams
        | RuntimeFnId::StreamContextSetDefault
        | RuntimeFnId::StreamContextSetOption
        | RuntimeFnId::StreamContextSetParams
        | RuntimeFnId::StreamCopyToStream
        | RuntimeFnId::StreamFilterAppend
        | RuntimeFnId::StreamFilterPrepend
        | RuntimeFnId::StreamFilterRegister
        | RuntimeFnId::StreamFilterRemove
        | RuntimeFnId::StreamGetContents
        | RuntimeFnId::StreamGetFilters
        | RuntimeFnId::StreamGetLine
        | RuntimeFnId::StreamGetMetaData
        | RuntimeFnId::StreamGetTransports
        | RuntimeFnId::StreamGetWrappers
        | RuntimeFnId::StreamIsLocal
        | RuntimeFnId::StreamIsatty
        | RuntimeFnId::StreamResolveIncludePath
        | RuntimeFnId::StreamSelect
        | RuntimeFnId::StreamSetBlocking
        | RuntimeFnId::StreamSetChunkSize
        | RuntimeFnId::StreamSetReadBuffer
        | RuntimeFnId::StreamSetTimeout
        | RuntimeFnId::StreamSetWriteBuffer
        | RuntimeFnId::StreamSocketAccept
        | RuntimeFnId::StreamSocketClient
        | RuntimeFnId::StreamSocketEnableCrypto
        | RuntimeFnId::StreamSocketGetName
        | RuntimeFnId::StreamSocketPair
        | RuntimeFnId::StreamSocketRecvfrom
        | RuntimeFnId::StreamSocketSendto
        | RuntimeFnId::StreamSocketServer
        | RuntimeFnId::StreamSocketShutdown
        | RuntimeFnId::StreamSupportsLock
        | RuntimeFnId::StreamWrapperRegister
        | RuntimeFnId::StreamWrapperRestore
        | RuntimeFnId::StreamWrapperUnregister
        | RuntimeFnId::Symlink
        | RuntimeFnId::SysGetTempDir
        | RuntimeFnId::Tempnam
        | RuntimeFnId::Tmpfile
        | RuntimeFnId::Touch
        | RuntimeFnId::Umask
        | RuntimeFnId::Unlink
        | RuntimeFnId::VarDump
        | RuntimeFnId::Vfprintf
        | RuntimeFnId::Abs
        | RuntimeFnId::Acos
        | RuntimeFnId::Asin
        | RuntimeFnId::Atan
        | RuntimeFnId::Atan2
        | RuntimeFnId::Ceil
        | RuntimeFnId::Clamp
        | RuntimeFnId::Cos
        | RuntimeFnId::Cosh
        | RuntimeFnId::Deg2rad
        | RuntimeFnId::Exp
        | RuntimeFnId::Fdiv
        | RuntimeFnId::Floor
        | RuntimeFnId::Fmod
        | RuntimeFnId::Hypot
        | RuntimeFnId::Intdiv
        | RuntimeFnId::Log
        | RuntimeFnId::Log10
        | RuntimeFnId::Log2
        | RuntimeFnId::Max
        | RuntimeFnId::Min
        | RuntimeFnId::MtRand
        | RuntimeFnId::Pi
        | RuntimeFnId::Pow
        | RuntimeFnId::Rad2deg
        | RuntimeFnId::Rand
        | RuntimeFnId::RandomInt
        | RuntimeFnId::Round
        | RuntimeFnId::Sin
        | RuntimeFnId::Sinh
        | RuntimeFnId::Sqrt
        | RuntimeFnId::Tan
        | RuntimeFnId::Tanh
        | RuntimeFnId::ElephcPtrIsNull
        | RuntimeFnId::ElephcPtrReadString
        | RuntimeFnId::ElephcPtrWriteString
        | RuntimeFnId::BufferFree
        | RuntimeFnId::BufferLen
        | RuntimeFnId::Ptr
        | RuntimeFnId::PtrGet
        | RuntimeFnId::PtrIsNull
        | RuntimeFnId::PtrNull
        | RuntimeFnId::PtrOffset
        | RuntimeFnId::PtrRead16
        | RuntimeFnId::PtrRead32
        | RuntimeFnId::PtrRead8
        | RuntimeFnId::PtrReadString
        | RuntimeFnId::PtrSet
        | RuntimeFnId::PtrSizeof
        | RuntimeFnId::PtrWrite16
        | RuntimeFnId::PtrWrite32
        | RuntimeFnId::PtrWrite8
        | RuntimeFnId::PtrWriteString
        | RuntimeFnId::ZvalFree
        | RuntimeFnId::ZvalPack
        | RuntimeFnId::ZvalType
        | RuntimeFnId::ZvalUnpack
        | RuntimeFnId::IteratorApply
        | RuntimeFnId::IteratorCount
        | RuntimeFnId::IteratorToArray
        | RuntimeFnId::SplAutoload
        | RuntimeFnId::SplAutoloadCall
        | RuntimeFnId::SplAutoloadExtensions
        | RuntimeFnId::SplAutoloadFunctions
        | RuntimeFnId::SplAutoloadRegister
        | RuntimeFnId::SplAutoloadUnregister
        | RuntimeFnId::SplClasses
        | RuntimeFnId::SplObjectHash
        | RuntimeFnId::SplObjectId
        | RuntimeFnId::Chop
        | RuntimeFnId::Chr
        | RuntimeFnId::Crc32
        | RuntimeFnId::CtypeAlnum
        | RuntimeFnId::CtypeAlpha
        | RuntimeFnId::CtypeDigit
        | RuntimeFnId::CtypeSpace
        | RuntimeFnId::Explode
        | RuntimeFnId::GraphemeStrrev
        | RuntimeFnId::Gzcompress
        | RuntimeFnId::Gzdeflate
        | RuntimeFnId::Gzinflate
        | RuntimeFnId::Gzuncompress
        | RuntimeFnId::Hash
        | RuntimeFnId::HashAlgos
        | RuntimeFnId::HashCopy
        | RuntimeFnId::HashEquals
        | RuntimeFnId::HashFinal
        | RuntimeFnId::HashHmac
        | RuntimeFnId::HashInit
        | RuntimeFnId::HashUpdate
        | RuntimeFnId::Htmlentities
        | RuntimeFnId::Htmlspecialchars
        | RuntimeFnId::Implode
        | RuntimeFnId::InetNtop
        | RuntimeFnId::InetPton
        | RuntimeFnId::Ip2long
        | RuntimeFnId::Lcfirst
        | RuntimeFnId::Long2ip
        | RuntimeFnId::Ltrim
        | RuntimeFnId::MbEregMatch
        | RuntimeFnId::MbStrlen
        | RuntimeFnId::Md5
        | RuntimeFnId::NumberFormat
        | RuntimeFnId::Ord
        | RuntimeFnId::Printf
        | RuntimeFnId::Rtrim
        | RuntimeFnId::Sha1
        | RuntimeFnId::Sprintf
        | RuntimeFnId::Sscanf
        | RuntimeFnId::StrContains
        | RuntimeFnId::StrEndsWith
        | RuntimeFnId::StrIreplace
        | RuntimeFnId::StrPad
        | RuntimeFnId::StrRepeat
        | RuntimeFnId::StrReplace
        | RuntimeFnId::StrSplit
        | RuntimeFnId::StrStartsWith
        | RuntimeFnId::Strcasecmp
        | RuntimeFnId::Strcmp
        | RuntimeFnId::Strpos
        | RuntimeFnId::Strrpos
        | RuntimeFnId::Strstr
        | RuntimeFnId::Substr
        | RuntimeFnId::SubstrReplace
        | RuntimeFnId::Trim
        | RuntimeFnId::Ucfirst
        | RuntimeFnId::Ucwords
        | RuntimeFnId::Vprintf
        | RuntimeFnId::Vsprintf
        | RuntimeFnId::Wordwrap
        | RuntimeFnId::ElephcGmmktimeRaw
        | RuntimeFnId::ElephcMktimeRaw
        | RuntimeFnId::ElephcStrtotimeRaw
        | RuntimeFnId::Checkdate
        | RuntimeFnId::ClassAttributeArgs
        | RuntimeFnId::ClassAttributeNames
        | RuntimeFnId::ClassGetAttributes
        | RuntimeFnId::Date
        | RuntimeFnId::DateDefaultTimezoneGet
        | RuntimeFnId::DateDefaultTimezoneSet
        | RuntimeFnId::Define
        | RuntimeFnId::Defined
        | RuntimeFnId::Exec
        | RuntimeFnId::ExtensionLoaded
        | RuntimeFnId::Getdate
        | RuntimeFnId::Getenv
        | RuntimeFnId::Gmdate
        | RuntimeFnId::Gmmktime
        | RuntimeFnId::Header
        | RuntimeFnId::Hrtime
        | RuntimeFnId::HttpResponseCode
        | RuntimeFnId::JsonDecode
        | RuntimeFnId::JsonEncode
        | RuntimeFnId::JsonLastError
        | RuntimeFnId::JsonLastErrorMsg
        | RuntimeFnId::JsonValidate
        | RuntimeFnId::Localtime
        | RuntimeFnId::Microtime
        | RuntimeFnId::Mktime
        | RuntimeFnId::Passthru
        | RuntimeFnId::PhpUname
        | RuntimeFnId::Phpversion
        | RuntimeFnId::PregMatch
        | RuntimeFnId::PregMatchAll
        | RuntimeFnId::PregReplace
        | RuntimeFnId::PregSplit
        | RuntimeFnId::Putenv
        | RuntimeFnId::Serialize
        | RuntimeFnId::ShellExec
        | RuntimeFnId::Sleep
        | RuntimeFnId::Strtotime
        | RuntimeFnId::System
        | RuntimeFnId::Time
        | RuntimeFnId::Unserialize
        | RuntimeFnId::Usleep
        | RuntimeFnId::GetResourceId
        | RuntimeFnId::GetResourceType
        | RuntimeFnId::Gettype
        | RuntimeFnId::IsCallable
        | RuntimeFnId::IsFinite
        | RuntimeFnId::IsInfinite
        | RuntimeFnId::IsNan
        | RuntimeFnId::IsNumeric
        | RuntimeFnId::Settype
        => false,
    }
}

/// Returns the stable name of every unary-string runtime variant.
fn unary_string_name(target: UnaryStringRuntime) -> &'static str {
    match target {
        UnaryStringRuntime::AddSlashes => "string.add_slashes",
        UnaryStringRuntime::Base64Decode => "string.base64_decode",
        UnaryStringRuntime::Base64Encode => "string.base64_encode",
        UnaryStringRuntime::BinToHex => "string.bin_to_hex",
        UnaryStringRuntime::HexToBin => "string.hex_to_bin",
        UnaryStringRuntime::HtmlEntityDecode => "string.html_entity_decode",
        UnaryStringRuntime::NlToBr => "string.nl_to_br",
        UnaryStringRuntime::RawUrlDecode => "string.raw_url_decode",
        UnaryStringRuntime::RawUrlEncode => "string.raw_url_encode",
        UnaryStringRuntime::StripSlashes => "string.strip_slashes",
        UnaryStringRuntime::StrReverse => "string.reverse",
        UnaryStringRuntime::StrToLower => "string.to_lower",
        UnaryStringRuntime::StrToUpper => "string.to_upper",
        UnaryStringRuntime::UrlDecode => "string.url_decode",
        UnaryStringRuntime::UrlEncode => "string.url_encode",
    }
}

/// Returns whether an EIR terminator has an active WASM lowering.
fn terminator_is_supported(terminator: &Terminator) -> bool {
    match terminator {
        Terminator::Br { .. }
        | Terminator::CondBr { .. }
        | Terminator::Switch { .. }
        | Terminator::Return { .. }
        | Terminator::Unreachable => true,
        Terminator::Throw { .. }
        | Terminator::Fatal { .. }
        | Terminator::GeneratorSuspend { .. } => false,
    }
}

/// Returns the stable diagnostic name for every EIR terminator.
fn terminator_name(terminator: &Terminator) -> &'static str {
    match terminator {
        Terminator::Br { .. } => "br",
        Terminator::CondBr { .. } => "cond_br",
        Terminator::Switch { .. } => "switch",
        Terminator::Return { .. } => "return",
        Terminator::Throw { .. } => "throw",
        Terminator::Fatal { .. } => "fatal",
        Terminator::GeneratorSuspend { .. } => "generator_suspend",
        Terminator::Unreachable => "unreachable",
    }
}

/// Returns whether an EIR opcode has an active WASM dispatch.
fn op_is_supported(op: Op) -> bool {
    match op {
        Op::ConstI64
        | Op::ConstF64
        | Op::ConstStr
        | Op::ConstNull
        | Op::ConstBool
        | Op::LoadLocal
        | Op::StoreLocal
        | Op::LoadRefCell
        | Op::StoreRefCell
        | Op::PromoteLocalRefCell
        | Op::AliasLocalRefCell
        | Op::ReleaseLocalRefCell
        | Op::LoadGlobal
        | Op::IAdd
        | Op::ISub
        | Op::IMul
        | Op::ICheckedAdd
        | Op::ICheckedSub
        | Op::ICheckedMul
        | Op::IDiv
        | Op::ISDiv
        | Op::ISMod
        | Op::INeg
        | Op::IBitAnd
        | Op::IBitOr
        | Op::IBitXor
        | Op::IBitNot
        | Op::IShl
        | Op::IShrA
        | Op::FAdd
        | Op::FSub
        | Op::FMul
        | Op::FDiv
        | Op::FNeg
        | Op::ICmp
        | Op::FCmp
        | Op::IsNull
        | Op::IsTruthy
        | Op::InstanceOf
        | Op::IToF
        | Op::FToI
        | Op::Cast
        | Op::MixedBox
        | Op::MixedTagOf
        | Op::StrConcat
        | Op::StrLen
        | Op::ConcatReset
        | Op::ArrayNew
        | Op::HashNew
        | Op::ArrayLen
        | Op::ArrayGet
        | Op::ArrayGetSilent
        | Op::HashGet
        | Op::ArraySet
        | Op::HashSet
        | Op::HashUnset
        | Op::ArrayPush
        | Op::ArrayToHash
        | Op::HashAppend
        | Op::ArrayUnion
        | Op::HashUnion
        | Op::ArrayHashUnion
        | Op::HashArrayUnion
        | Op::IterStart
        | Op::IterCurrentKey
        | Op::IterCurrentValue
        | Op::IterCurrentValueRef
        | Op::IterNext
        | Op::IterEnd
        | Op::ObjectNew
        | Op::PropGet
        | Op::PropSet
        | Op::NullsafePropGet
        | Op::NullsafeMethodCall
        | Op::MethodCall
        | Op::StaticMethodCall
        | Op::InstanceOfDynamic
        | Op::Call
        | Op::LanguageConstructCall
        | Op::RuntimeCall
        | Op::ClosureNew
        | Op::ClosureCapture
        | Op::ClosureCall
        | Op::FirstClassCallableNew
        | Op::CallableDescriptorInvoke
        | Op::EchoValue
        | Op::PrintValue
        | Op::Acquire
        | Op::Release
        | Op::Move
        | Op::Borrow
        | Op::Nop => true,
        Op::ConstClassName
        | Op::ConstEnumCase
        | Op::LoadCalledClassId
        | Op::DataAddr
        | Op::UnsetLocal
        | Op::ReleaseLocalSlot
        | Op::StoreGlobal
        | Op::LoadStaticLocal
        | Op::StoreStaticLocal
        | Op::InitStaticLocal
        | Op::LoadStaticProperty
        | Op::StoreStaticProperty
        | Op::LoadReflectionStaticProperty
        | Op::StoreReflectionStaticProperty
        | Op::ReflectionStaticPropertyInitialized
        | Op::IPow
        | Op::FPow
        | Op::MixedNumericBinop
        | Op::StrEq
        | Op::StrCmp
        | Op::StrLooseEq
        | Op::StrictEq
        | Op::StrictNotEq
        | Op::LooseEq
        | Op::LooseNotEq
        | Op::Spaceship
        | Op::TypePredicate
        | Op::IsEmpty
        | Op::IToStr
        | Op::FToStr
        | Op::BoolToStr
        | Op::StrToI
        | Op::StrToF
        | Op::StrToNumber
        | Op::ResourceToStr
        | Op::InvokerRefArg
        | Op::MixedUnbox
        | Op::ArrayToMixed
        | Op::HashToMixed
        | Op::MixedCastBool
        | Op::MixedCastInt
        | Op::MixedCastFloat
        | Op::MixedCastString
        | Op::StrPersist
        | Op::StrCharAt
        | Op::StrInterpolate
        | Op::WriteStrStdout
        | Op::HashLen
        | Op::HashGetSilent
        | Op::ArrayIsset
        | Op::HashIsset
        | Op::ArrayElemAddr
        | Op::MixedArrayAppend
        | Op::ArrayEnsureUnique
        | Op::HashEnsureUnique
        | Op::ArrayCloneShallow
        | Op::HashCloneShallow
        | Op::HashSpread
        | Op::ArraySetMixedKey
        | Op::ArrayGetMixedKey
        | Op::ArrayGetMixedKeySilent
        | Op::ArrayKeyExists
        | Op::OffsetExists
        | Op::OffsetUnset
        | Op::ListUnpack
        | Op::IteratorMethodCall
        | Op::SplRuntimeCall
        | Op::EvalObjectNew
        | Op::ObjectCloneShallow
        | Op::DynamicObjectNew
        | Op::DynamicObjectNewMixed
        | Op::DynamicObjectNewWithoutConstructorMixed
        | Op::PropInitialized
        | Op::LoadPropRefCell
        | Op::LoadArrayElemRefCell
        | Op::BindRefCellPtr
        | Op::DynamicPropGet
        | Op::DynamicPropSet
        | Op::MethodLookup
        | Op::EvalStaticMethodCall
        | Op::EnumBackingStringToInt
        | Op::EnumBackingMixedToInt
        | Op::ClassConstant
        | Op::ScopedConstantGet
        | Op::ClassAttrNames
        | Op::ClassAttrArgs
        | Op::ClassGetAttributes
        | Op::FunctionVariantCall
        | Op::ClosureBind
        | Op::EvalLiteralCall
        | Op::EvalScopeGet
        | Op::EvalScopeSet
        | Op::EvalFunctionCall
        | Op::EvalFunctionCallArray
        | Op::EvalFunctionExists
        | Op::EvalClassExists
        | Op::EvalConstantExists
        | Op::EvalConstantFetch
        | Op::ExternCall
        | Op::ExprCall
        | Op::CallableArrayNew
        | Op::PipeCall
        | Op::PtrCast
        | Op::PtrRead
        | Op::PtrWrite
        | Op::PtrReadString
        | Op::PtrWriteString
        | Op::PtrOffset
        | Op::PtrCheckNonnull
        | Op::BufferNew
        | Op::BufferLen
        | Op::BufferGet
        | Op::BufferSet
        | Op::BufferFree
        | Op::PackedFieldGet
        | Op::PackedFieldSet
        | Op::ExternGlobalLoad
        | Op::ExternGlobalStore
        | Op::WriteStdout
        | Op::VarDump
        | Op::PrintR
        | Op::ErrorSuppressBegin
        | Op::ErrorSuppressEnd
        | Op::Warn
        | Op::ThrowException
        | Op::ThrowError
        | Op::ThrowErrorValue
        | Op::TryPushHandler
        | Op::TryPopHandler
        | Op::CatchCurrent
        | Op::CatchBind
        | Op::FinallyEnter
        | Op::FinallyExit
        | Op::FiberRuntimeCall
        | Op::GeneratorNew
        | Op::GeneratorYield
        | Op::GeneratorYieldFrom
        | Op::GeneratorReturn
        | Op::IncludeOnceMark
        | Op::IncludeOnceGuard
        | Op::FunctionVariantMark
        | Op::FunctionVariantDispatch
        | Op::GcCollect
        | Op::EnsureOwned => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_module as validate_and_plan, LoweredWasmPlan, WasmError};
    use crate::codegen::platform::Target;
    use crate::codegen::Emit;
    use crate::ir::{
        Builder, DataId, Function, FunctionParam, Immediate, IrHeapKind, IrType, LocalKind, Module,
        Op, Ownership, RuntimeCallTarget, RuntimeFnId, Terminator,
    };
    use crate::span::Span;
    use crate::types::{ClassInfo, FunctionSig, PhpType};
    use std::collections::{HashMap, HashSet};

    /// Runs the production capability-and-planning gate for executable output.
    fn validate_module(module: &Module) -> Result<LoweredWasmPlan, WasmError> {
        validate_and_plan(module, Emit::Executable)
    }

    /// Builds the minimal resolved class metadata needed by WASM planning tests.
    fn minimal_class_info(class_id: u64) -> ClassInfo {
        ClassInfo {
            class_id,
            declaration_span: Span::dummy(),
            parent: None,
            is_abstract: false,
            is_final: false,
            is_readonly_class: false,
            allow_dynamic_properties: false,
            constants: HashMap::new(),
            constant_types: HashMap::new(),
            constant_visibilities: HashMap::new(),
            final_constants: HashSet::new(),
            attribute_names: Vec::new(),
            attribute_args: Vec::new(),
            method_attribute_names: HashMap::new(),
            method_attribute_args: HashMap::new(),
            property_attribute_names: HashMap::new(),
            property_attribute_args: HashMap::new(),
            constant_attribute_names: HashMap::new(),
            constant_attribute_args: HashMap::new(),
            used_traits: Vec::new(),
            trait_aliases: Vec::new(),
            properties: Vec::new(),
            property_offsets: HashMap::new(),
            property_declaring_classes: HashMap::new(),
            defaults: Vec::new(),
            property_visibilities: HashMap::new(),
            property_set_visibilities: HashMap::new(),
            declared_properties: HashSet::new(),
            property_declared_slots: Vec::new(),
            final_properties: HashSet::new(),
            readonly_properties: HashSet::new(),
            reference_properties: HashSet::new(),
            owned_reference_properties: HashSet::new(),
            promoted_properties: HashSet::new(),
            property_reference_slots: Vec::new(),
            abstract_properties: HashSet::new(),
            abstract_property_hooks: HashMap::new(),
            static_properties: Vec::new(),
            static_defaults: Vec::new(),
            static_property_declaring_classes: HashMap::new(),
            static_property_visibilities: HashMap::new(),
            declared_static_properties: HashSet::new(),
            final_static_properties: HashSet::new(),
            method_decls: Vec::new(),
            methods: HashMap::new(),
            static_methods: HashMap::new(),
            late_static_method_returns: HashMap::new(),
            late_static_static_method_returns: HashMap::new(),
            callable_method_return_sigs: HashMap::new(),
            callable_array_method_return_sigs: HashMap::new(),
            method_visibilities: HashMap::new(),
            final_methods: HashSet::new(),
            method_declaring_classes: HashMap::new(),
            method_impl_classes: HashMap::new(),
            vtable_methods: Vec::new(),
            vtable_slots: HashMap::new(),
            static_method_visibilities: HashMap::new(),
            final_static_methods: HashSet::new(),
            static_method_declaring_classes: HashMap::new(),
            static_method_impl_classes: HashMap::new(),
            static_vtable_methods: Vec::new(),
            static_vtable_slots: HashMap::new(),
            interfaces: Vec::new(),
            constructor_param_to_prop: Vec::new(),
        }
    }

    /// Builds a declared no-argument void method signature.
    fn void_signature() -> FunctionSig {
        FunctionSig {
            params: Vec::new(),
            param_type_exprs: Vec::new(),
            param_attributes: Vec::new(),
            defaults: Vec::new(),
            return_type: PhpType::Void,
            declared_return: true,
            by_ref_return: false,
            ref_params: Vec::new(),
            declared_params: Vec::new(),
            variadic: None,
            deprecation: None,
        }
    }

    /// Builds a declared one-integer-to-integer method signature.
    fn int_method_signature() -> FunctionSig {
        FunctionSig {
            params: vec![("value".to_string(), PhpType::Int)],
            param_type_exprs: Vec::new(),
            param_attributes: Vec::new(),
            defaults: Vec::new(),
            return_type: PhpType::Int,
            declared_return: true,
            by_ref_return: false,
            ref_params: vec![false],
            declared_params: vec![true],
            variadic: None,
            deprecation: None,
        }
    }

    /// Rejects a same-width callable parameter hidden behind an integer method
    /// signature before direct, static, or dynamic lowering can raw-copy it.
    #[test]
    fn method_body_contract_rejects_same_width_callable_parameter_drift() {
        let mut body =
            Function::new("C::f".to_string(), IrType::I64, PhpType::Int);
        body.params.push(FunctionParam {
            name: "called_class".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Int,
            by_ref: false,
            variadic: false,
        });
        body.params.push(FunctionParam {
            name: "value".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Callable,
            by_ref: false,
            variadic: false,
        });

        let issue = super::method_body_signature_shape_issue(
            &body,
            &int_method_signature(),
            IrType::I64,
        )
        .expect("same-width PHP metadata drift must be rejected");
        assert!(issue.contains("parameter #0"));
        assert!(issue.contains("Callable"));
    }

    /// Rejects a same-width callable return hidden behind an integer method
    /// signature before direct storage or dynamic Mixed boxing.
    #[test]
    fn method_body_contract_rejects_same_width_callable_return_drift() {
        let mut body =
            Function::new("C::f".to_string(), IrType::I64, PhpType::Callable);
        body.params.push(FunctionParam {
            name: "called_class".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Int,
            by_ref: false,
            variadic: false,
        });
        body.params.push(FunctionParam {
            name: "value".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Int,
            by_ref: false,
            variadic: false,
        });

        let issue = super::method_body_signature_shape_issue(
            &body,
            &int_method_signature(),
            IrType::I64,
        )
        .expect("same-width PHP return metadata drift must be rejected");
        assert!(issue.contains("return"));
        assert!(issue.contains("Callable"));
    }

    /// Builds one terminated void function suitable for collection-audit tests.
    fn void_function(name: &str) -> Function {
        let mut function = Function::new(name.to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            builder.terminate(Terminator::Return { value: None });
        }
        function
    }

    /// Builds a malformed branch from a Mixed cell into tagged-scalar storage.
    fn invalid_mixed_transfer_module() -> Module {
        let mut module = Module::new(Target::wasm());
        let mut function =
            Function::new("invalid_mixed_transfer".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block(
                "entry",
                vec![(IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed)],
            );
            let target = builder.create_named_block(
                "target",
                vec![(IrType::TaggedScalar, PhpType::TaggedScalar)],
            );
            builder.set_entry(entry);
            let mixed = builder.block_param(entry, 0);
            builder.position_at_end(entry);
            builder.terminate(Terminator::Br {
                target,
                args: vec![mixed],
            });
            builder.position_at_end(target);
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(function);
        module
    }

    /// Builds a module whose `ConstStr` references no real literal.
    fn invalid_const_str_module() -> Module {
        let mut module = Module::new(Target::wasm());
        let mut main = Function::new("main".to_string(), IrType::Void, PhpType::Void);
        main.flags.is_main = true;
        {
            let mut builder = Builder::new(&mut main);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let _ = builder.emit(
                Op::ConstStr,
                Vec::new(),
                Some(Immediate::Data(DataId::from_raw(99))),
                IrType::Str,
                PhpType::Str,
                Ownership::NonHeap,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(main);
        module
    }

    /// Builds a closure whose recorded capture count exceeds its parameters.
    fn invalid_capture_count_module() -> Module {
        let mut module = Module::new(Target::wasm());
        let mut closure = void_function("__eir_closure_invalid_capture_0");
        closure.flags.is_closure = true;
        closure.flags.closure_capture_count = 1;
        module.add_closure(closure);
        module
    }

    /// Builds class metadata with a destructor implementation that no class declares.
    fn stale_destructor_metadata_module() -> Module {
        let mut module = Module::new(Target::wasm());
        let mut class = minimal_class_info(1);
        class.method_impl_classes.insert(
            crate::names::php_symbol_key("__destruct"),
            "MissingDestructorOwner".to_string(),
        );
        module.class_infos.insert("Victim".to_string(), class);
        module
    }

    /// Builds an FCC target whose by-reference parameter cannot be wrapped.
    fn invalid_fcc_wrapper_module() -> Module {
        let mut module = Module::new(Target::wasm());
        let target_name = module.data.intern_string("bad_fcc");
        let mut target = Function::new("bad_fcc".to_string(), IrType::Void, PhpType::Void);
        target.params.push(FunctionParam {
            name: "value".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Int,
            by_ref: true,
            variadic: false,
        });
        target.add_local(
            Some("value".to_string()),
            IrType::I64,
            PhpType::Int,
            LocalKind::PhpLocal,
        );
        {
            let mut builder = Builder::new(&mut target);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(target);

        let mut main = Function::new("main".to_string(), IrType::Void, PhpType::Void);
        main.flags.is_main = true;
        {
            let mut builder = Builder::new(&mut main);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let _ = builder.emit(
                Op::FirstClassCallableNew,
                Vec::new(),
                Some(Immediate::Data(target_name)),
                IrType::I64,
                PhpType::Callable,
                Ownership::Owned,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(main);
        module
    }

    /// Builds a dynamic-looking closure call whose body has an unsupported
    /// visible Mixed parameter; `ClosureNew` must reject it before planning.
    fn invalid_masked_closure_wrapper_param_module() -> Module {
        let mut module = Module::new(Target::wasm());
        let closure_name = "__eir_closure_bad_visible_0";
        let closure_name_id = module.data.intern_string(closure_name);
        let mut closure =
            Function::new(closure_name.to_string(), IrType::I64, PhpType::Int);
        closure.flags.is_closure = true;
        closure.params.push(FunctionParam {
            name: "value".to_string(),
            ir_type: IrType::Heap(IrHeapKind::Mixed),
            php_type: PhpType::Mixed,
            by_ref: false,
            variadic: false,
        });
        {
            let mut builder = Builder::new(&mut closure);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let result = builder.emit_const_i64(1);
            builder.terminate(Terminator::Return {
                value: Some(result),
            });
        }
        module.add_closure(closure);

        let mut main = Function::new("main".to_string(), IrType::Void, PhpType::Void);
        main.flags.is_main = true;
        let callable_slot = main.add_local(
            Some("callable".to_string()),
            IrType::I64,
            PhpType::Callable,
            LocalKind::PhpLocal,
        );
        {
            let mut builder = Builder::new(&mut main);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let descriptor = builder
                .emit(
                    Op::ClosureNew,
                    Vec::new(),
                    Some(Immediate::Data(closure_name_id)),
                    IrType::I64,
                    PhpType::Callable,
                    Ownership::Owned,
                )
                .expect("closure descriptor");
            builder.emit_store_local(callable_slot, descriptor);
            let masked =
                builder.emit_load_local(callable_slot, IrType::I64, PhpType::Callable);
            let argument = builder.emit_const_i64(1);
            let _ = builder.emit(
                Op::ClosureCall,
                vec![masked, argument],
                None,
                IrType::I64,
                PhpType::Int,
                Ownership::NonHeap,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(main);
        module
    }

    /// Builds a closure whose tagged-scalar return cannot be boxed by the
    /// generated callable wrapper.
    fn invalid_closure_wrapper_return_module() -> Module {
        let mut module = Module::new(Target::wasm());
        let closure_name = "__eir_closure_bad_return_0";
        let closure_name_id = module.data.intern_string(closure_name);
        let mut closure = Function::new(
            closure_name.to_string(),
            IrType::TaggedScalar,
            PhpType::TaggedScalar,
        );
        closure.flags.is_closure = true;
        module.add_closure(closure);

        let mut main = Function::new("main".to_string(), IrType::Void, PhpType::Void);
        main.flags.is_main = true;
        {
            let mut builder = Builder::new(&mut main);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let _ = builder.emit(
                Op::ClosureNew,
                Vec::new(),
                Some(Immediate::Data(closure_name_id)),
                IrType::I64,
                PhpType::Callable,
                Ownership::Owned,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(main);
        module
    }

    /// Builds closure calls whose Mixed and null-sentinel arguments are outside
    /// the exact boxing surface used by `lower_closure_call`.
    fn invalid_closure_call_argument_boxing_module() -> Module {
        let mut module = Module::new(Target::wasm());
        let mut main = Function::new("main".to_string(), IrType::Void, PhpType::Void);
        main.flags.is_main = true;
        {
            let mut builder = Builder::new(&mut main);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let callable = builder
                .emit(
                    Op::ConstI64,
                    Vec::new(),
                    Some(Immediate::I64(0)),
                    IrType::I64,
                    PhpType::Callable,
                    Ownership::NonHeap,
                )
                .expect("dynamic callable placeholder");
            let integer = builder.emit_const_i64(1);
            let mixed = builder
                .emit(
                    Op::MixedBox,
                    vec![integer],
                    None,
                    IrType::Heap(IrHeapKind::Mixed),
                    PhpType::Mixed,
                    Ownership::Owned,
                )
                .expect("mixed argument");
            let null = builder.emit_const_null();
            for argument in [mixed, null] {
                let _ = builder.emit(
                    Op::ClosureCall,
                    vec![callable, argument],
                    None,
                    IrType::Void,
                    PhpType::Void,
                    Ownership::NonHeap,
                );
            }
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(main);
        module
    }

    /// Builds a branch whose argument count differs from the target parameters.
    fn invalid_branch_arity_module() -> Module {
        let mut module = Module::new(Target::wasm());
        let mut function =
            Function::new("invalid_branch_arity".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            let target =
                builder.create_named_block("target", vec![(IrType::I64, PhpType::Int)]);
            builder.set_entry(entry);
            builder.position_at_end(entry);
            builder.terminate(Terminator::Br {
                target,
                args: Vec::new(),
            });
            builder.position_at_end(target);
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(function);
        module
    }

    /// Builds one valid module combining class/destructor, closure, and FCC planning.
    fn combined_plan_module() -> Module {
        let mut module = Module::new(Target::wasm());
        let destruct_key = crate::names::php_symbol_key("__destruct");
        let mut class = minimal_class_info(1);
        class
            .methods
            .insert(destruct_key.clone(), void_signature());
        class
            .method_impl_classes
            .insert(destruct_key, "PlanClass".to_string());
        module.class_infos.insert("PlanClass".to_string(), class);

        let mut destructor = Function::new(
            "PlanClass::__destruct".to_string(),
            IrType::Void,
            PhpType::Void,
        );
        destructor.flags.is_method = true;
        destructor.params.push(FunctionParam {
            name: "this".to_string(),
            ir_type: IrType::Heap(IrHeapKind::Object),
            php_type: PhpType::Object("PlanClass".to_string()),
            by_ref: false,
            variadic: false,
        });
        destructor.add_local(
            Some("this".to_string()),
            IrType::Heap(IrHeapKind::Object),
            PhpType::Object("PlanClass".to_string()),
            LocalKind::PhpLocal,
        );
        {
            let mut builder = Builder::new(&mut destructor);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            builder.terminate(Terminator::Return { value: None });
        }
        module.class_methods.push(destructor);

        let mut closure = void_function("__eir_closure_plan_combo_0");
        closure.flags.is_closure = true;
        module.add_closure(closure);

        let fcc_name = module.data.intern_string("plan_fcc");
        module.add_function(void_function("plan_fcc"));

        let mut main = Function::new("main".to_string(), IrType::Void, PhpType::Void);
        main.flags.is_main = true;
        {
            let mut builder = Builder::new(&mut main);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let _ = builder.emit(
                Op::FirstClassCallableNew,
                Vec::new(),
                Some(Immediate::Data(fcc_name)),
                IrType::I64,
                PhpType::Callable,
                Ownership::Owned,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(main);
        module
    }

    /// Adds a minimal closure whose only capture parameter is an int passed by
    /// reference, returning the interned closure-name data id.
    fn add_by_ref_int_closure(module: &mut Module, name: &str) -> crate::ir::DataId {
        let name_id = module.data.intern_string(name);
        let mut closure = Function::new(name.to_string(), IrType::Void, PhpType::Void);
        closure.flags.is_closure = true;
        closure.flags.closure_capture_count = 1;
        closure.params.push(FunctionParam {
            name: "captured".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Int,
            by_ref: true,
            variadic: false,
        });
        closure.add_local(
            Some("captured".to_string()),
            IrType::I64,
            PhpType::Int,
            LocalKind::PhpLocal,
        );
        {
            let mut builder = Builder::new(&mut closure);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_closure(closure);
        name_id
    }

    /// Emits one closure descriptor creation from `capture` in the positioned
    /// builder and returns the callable value.
    fn emit_by_ref_closure_new(
        builder: &mut Builder<'_>,
        capture: crate::ir::ValueId,
        name_id: crate::ir::DataId,
    ) -> crate::ir::ValueId {
        builder
            .emit(
                Op::ClosureNew,
                vec![capture],
                Some(Immediate::Data(name_id)),
                IrType::I64,
                PhpType::Callable,
                Ownership::Owned,
            )
            .expect("closure descriptor")
    }

    /// Verifies capability validation rejects a direct call whose lowered
    /// operand count cannot satisfy the resolved target's exact WAT signature.
    #[test]
    fn direct_call_shape_rejects_arity_mismatch_before_lowering() {
        let mut module = Module::new(Target::wasm());
        let name = module.data.intern_function_name("Target");
        let mut target = Function::new("target".to_string(), IrType::Void, PhpType::Void);
        target.params.push(FunctionParam {
            name: "value".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Int,
            by_ref: false,
            variadic: false,
        });
        module.add_function(target);
        let owner = void_function("owner");
        let call = crate::ir::Instruction::new(
            Op::Call,
            Vec::new(),
            Some(Immediate::Data(name)),
            None,
            IrType::Void,
            PhpType::Void,
            Ownership::NonHeap,
            Op::Call.default_effects(),
            None,
        );

        let issue = super::direct_call_shape_issue(
            &module,
            &owner,
            &call,
            &super::RefCellProvenance::default(),
        )
        .expect("arity mismatch");

        assert!(issue.contains("expects 1 lowered operands, got 0"), "{issue}");
    }

    /// Verifies a concrete heap pointer cannot cross a direct-call boundary
    /// when its PHP payload layout differs from the resolved parameter.
    #[test]
    fn direct_call_shape_rejects_array_payload_type_mismatch() {
        let mut module = Module::new(Target::wasm());
        let name = module.data.intern_function_name("consume_strings");
        let mut target =
            Function::new("consume_strings".to_string(), IrType::Void, PhpType::Void);
        target.params.push(FunctionParam {
            name: "values".to_string(),
            ir_type: IrType::Heap(IrHeapKind::Array),
            php_type: PhpType::Array(Box::new(PhpType::Str)),
            by_ref: false,
            variadic: false,
        });
        module.add_function(target);
        let mut owner = Function::new("owner".to_string(), IrType::Void, PhpType::Void);
        let argument;
        {
            let mut builder = Builder::new(&mut owner);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            argument = builder
                .emit(
                    Op::ArrayNew,
                    Vec::new(),
                    Some(Immediate::I64(0)),
                    IrType::Heap(IrHeapKind::Array),
                    PhpType::Array(Box::new(PhpType::Int)),
                    Ownership::Owned,
                )
                .expect("array argument");
            builder.terminate(Terminator::Return { value: None });
        }
        let call = crate::ir::Instruction::new(
            Op::Call,
            vec![argument],
            Some(Immediate::Data(name)),
            None,
            IrType::Void,
            PhpType::Void,
            Ownership::NonHeap,
            Op::Call.default_effects(),
            None,
        );

        let issue = super::direct_call_shape_issue(
            &module,
            &owner,
            &call,
            &super::collect_ref_cell_provenance(&owner),
        )
        .expect("array payload mismatch");

        assert!(issue.contains("unsupported wasm value transfer"), "{issue}");
    }

    /// Verifies a direct call cannot route a Mixed cell through the generic
    /// integer cast path when the callee expects a callable descriptor.
    #[test]
    fn direct_call_shape_rejects_generic_mixed_to_callable_unboxing() {
        let mut module = Module::new(Target::wasm());
        let name = module.data.intern_function_name("invoke");
        let mut target = Function::new("invoke".to_string(), IrType::Void, PhpType::Void);
        target.params.push(FunctionParam {
            name: "callback".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Callable,
            by_ref: false,
            variadic: false,
        });
        module.add_function(target);
        let mut owner = Function::new("owner".to_string(), IrType::Void, PhpType::Void);
        let argument;
        {
            let mut builder = Builder::new(&mut owner);
            let entry = builder.create_named_block(
                "entry",
                vec![(IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed)],
            );
            builder.set_entry(entry);
            argument = builder.block_param(entry, 0);
            builder.position_at_end(entry);
            builder.terminate(Terminator::Return { value: None });
        }
        let call = crate::ir::Instruction::new(
            Op::Call,
            vec![argument],
            Some(Immediate::Data(name)),
            None,
            IrType::Void,
            PhpType::Void,
            Ownership::NonHeap,
            Op::Call.default_effects(),
            None,
        );

        let issue = super::direct_call_shape_issue(
            &module,
            &owner,
            &call,
            &super::RefCellProvenance::default(),
        )
        .expect("generic Mixed-to-callable transfer must be rejected");

        assert!(issue.contains("unboxing a Mixed cell"), "{issue}");
        assert!(issue.contains("Callable"), "{issue}");
    }

    /// Verifies a malformed `LoadRefCell` cannot reach direct-call emission
    /// unless the slot was registered as an owned or borrowed ref binding.
    #[test]
    fn direct_call_shape_rejects_unregistered_ref_cell_operand() {
        let mut module = Module::new(Target::wasm());
        let name = module.data.intern_function_name("mutate");
        let mut target = Function::new("mutate".to_string(), IrType::Void, PhpType::Void);
        target.params.push(FunctionParam {
            name: "value".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Int,
            by_ref: true,
            variadic: false,
        });
        module.add_function(target);
        let mut owner = Function::new("owner".to_string(), IrType::Void, PhpType::Void);
        let slot = owner.add_local(
            Some("value".to_string()),
            IrType::I64,
            PhpType::Int,
            LocalKind::PhpLocal,
        );
        let argument;
        {
            let mut builder = Builder::new(&mut owner);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            argument = builder
                .emit(
                    Op::LoadRefCell,
                    Vec::new(),
                    Some(Immediate::LocalSlot(slot)),
                    IrType::I64,
                    PhpType::Int,
                    Ownership::NonHeap,
                )
                .expect("malformed ref-cell load");
            builder.terminate(Terminator::Return { value: None });
        }
        let call = crate::ir::Instruction::new(
            Op::Call,
            vec![argument],
            Some(Immediate::Data(name)),
            None,
            IrType::Void,
            PhpType::Void,
            Ownership::NonHeap,
            Op::Call.default_effects(),
            None,
        );

        let issue = super::direct_call_shape_issue(
            &module,
            &owner,
            &call,
            &super::collect_ref_cell_provenance(&owner),
        )
        .expect("unregistered ref cell");

        assert!(issue.contains("unregistered ref-cell local"), "{issue}");
    }

    /// Verifies a by-reference `LoadLocal` cannot name a slot absent from the
    /// function's local table, even when its result metadata matches the callee.
    #[test]
    fn direct_call_shape_rejects_missing_fresh_local_slot() {
        let mut module = Module::new(Target::wasm());
        let name = module.data.intern_function_name("mutate_missing");
        let mut target =
            Function::new("mutate_missing".to_string(), IrType::Void, PhpType::Void);
        target.params.push(FunctionParam {
            name: "value".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Int,
            by_ref: true,
            variadic: false,
        });
        module.add_function(target);
        let mut owner = Function::new("owner".to_string(), IrType::Void, PhpType::Void);
        let missing = crate::ir::LocalSlotId::from_raw(99);
        let argument;
        {
            let mut builder = Builder::new(&mut owner);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            argument = builder
                .emit(
                    Op::LoadLocal,
                    Vec::new(),
                    Some(Immediate::LocalSlot(missing)),
                    IrType::I64,
                    PhpType::Int,
                    Ownership::NonHeap,
                )
                .expect("malformed local load");
            builder.terminate(Terminator::Return { value: None });
        }
        let call = crate::ir::Instruction::new(
            Op::Call,
            vec![argument],
            Some(Immediate::Data(name)),
            None,
            IrType::Void,
            PhpType::Void,
            Ownership::NonHeap,
            Op::Call.default_effects(),
            None,
        );

        let issue = super::direct_call_shape_issue(
            &module,
            &owner,
            &call,
            &super::RefCellProvenance::default(),
        )
        .expect("missing local must fail");

        assert!(issue.contains("local#99 is missing"), "{issue}");
    }

    /// Verifies both fresh and already-bound by-reference sources reject local
    /// metadata that differs from their loaded value and callee parameter.
    #[test]
    fn by_ref_sources_require_exact_slot_value_parameter_metadata() {
        let mut owner = Function::new("owner".to_string(), IrType::Void, PhpType::Void);
        let slot = owner.add_local(
            Some("value".to_string()),
            IrType::I64,
            PhpType::Callable,
            LocalKind::PhpLocal,
        );
        let fresh;
        let bound;
        {
            let mut builder = Builder::new(&mut owner);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            fresh = builder.emit_load_local(slot, IrType::I64, PhpType::Int);
            bound = builder
                .emit(
                    Op::LoadRefCell,
                    Vec::new(),
                    Some(Immediate::LocalSlot(slot)),
                    IrType::I64,
                    PhpType::Int,
                    Ownership::NonHeap,
                )
                .expect("ref-cell load");
            builder.terminate(Terminator::Return { value: None });
        }
        let parameter = FunctionParam {
            name: "value".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Int,
            by_ref: true,
            variadic: false,
        };

        for (value_id, source) in [
            (fresh, super::ByRefSource::FreshLocal(slot)),
            (
                bound,
                super::ByRefSource::AlreadyRefBound(slot.as_raw()),
            ),
        ] {
            let value = owner.value(value_id).expect("loaded value");
            let issue = super::by_ref_source_shape_issue(&owner, value, &parameter, source)
                .expect("metadata drift must fail");
            assert!(issue.contains("must match exactly"), "{issue}");
            assert!(issue.contains("Callable"), "{issue}");
        }
    }

    /// Verifies a by-reference parameter's borrowed cell can be forwarded to
    /// another by-reference callee without inventing a local owner.
    #[test]
    fn direct_call_shape_accepts_registered_borrowed_ref_cell_operand() {
        let mut module = Module::new(Target::wasm());
        let name = module.data.intern_function_name("forwarded_mutation");
        let mut target =
            Function::new("forwarded_mutation".to_string(), IrType::Void, PhpType::Void);
        target.params.push(FunctionParam {
            name: "value".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Int,
            by_ref: true,
            variadic: false,
        });
        module.add_function(target);
        let mut owner = Function::new("owner".to_string(), IrType::Void, PhpType::Void);
        owner.params.push(FunctionParam {
            name: "value".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Int,
            by_ref: true,
            variadic: false,
        });
        let slot = owner.add_local(
            Some("value".to_string()),
            IrType::I64,
            PhpType::Int,
            LocalKind::PhpLocal,
        );
        let argument;
        {
            let mut builder = Builder::new(&mut owner);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            argument = builder
                .emit(
                    Op::LoadRefCell,
                    Vec::new(),
                    Some(Immediate::LocalSlot(slot)),
                    IrType::I64,
                    PhpType::Int,
                    Ownership::NonHeap,
                )
                .expect("borrowed ref-cell load");
            builder.terminate(Terminator::Return { value: None });
        }
        let call = crate::ir::Instruction::new(
            Op::Call,
            vec![argument],
            Some(Immediate::Data(name)),
            None,
            IrType::Void,
            PhpType::Void,
            Ownership::NonHeap,
            Op::Call.default_effects(),
            None,
        );

        let issue = super::direct_call_shape_issue(
            &module,
            &owner,
            &call,
            &super::collect_ref_cell_provenance(&owner),
        );

        assert!(issue.is_none(), "{issue:?}");
    }

    /// Verifies descriptor invocation rejects an indexed array whose elements
    /// are not Mixed cells, preventing the wrapper from interpreting 8-byte
    /// integer slots as 16-byte cell slots.
    #[test]
    fn descriptor_invoke_shape_rejects_non_mixed_argument_array() {
        let module = Module::new(Target::wasm());
        let mut owner = Function::new("owner".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut owner);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let callable = builder
                .emit(
                    Op::ConstI64,
                    Vec::new(),
                    Some(Immediate::I64(0)),
                    IrType::I64,
                    PhpType::Callable,
                    Ownership::NonHeap,
                )
                .expect("callable value");
            let arguments = builder
                .emit(
                    Op::ArrayNew,
                    Vec::new(),
                    Some(Immediate::I64(0)),
                    IrType::Heap(IrHeapKind::Array),
                    PhpType::Array(Box::new(PhpType::Int)),
                    Ownership::Owned,
                )
                .expect("argument array");
            let _ = builder.emit(
                Op::CallableDescriptorInvoke,
                vec![callable, arguments],
                None,
                IrType::Void,
                PhpType::Void,
                Ownership::NonHeap,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        let invoke = owner
            .instructions
            .iter()
            .find(|instruction| instruction.op == Op::CallableDescriptorInvoke)
            .expect("descriptor invoke");

        let issue = super::callable_descriptor_invoke_shape_issue(&module, &owner, invoke)
            .expect("array<int> must be rejected");

        assert!(issue.contains("array<mixed>"), "{issue}");
    }

    /// Verifies validation returns the exact lowered plan for an accepted module.
    #[test]
    fn accepted_module_returns_an_assemblable_plan() {
        let mut module = Module::new(Target::wasm());
        let mut main = void_function("main");
        main.flags.is_main = true;
        module.add_function(main);

        let wat = validate_module(&module)
            .expect("the capability gate should return the exact plan")
            .into_wat();

        assert!(wat.contains("(export \"_start\")"), "{wat}");
        let bytes =
            ::wat::parse_str(&wat).unwrap_or_else(|error| panic!("WAT did not assemble: {error}"));
        wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::WASM3)
            .validate_all(&bytes)
            .unwrap_or_else(|error| panic!("WASM did not validate: {error}"));
    }

    /// Verifies one accepted plan combines classes, destructors, closures, and FCC.
    #[test]
    fn accepted_combined_module_plans_every_dispatch_surface() {
        let wat = validate_module(&combined_plan_module())
            .expect("the combined capability surface should produce one exact plan")
            .into_wat();
        let expected_symbols = [
            super::super::symbols::method_symbol("PlanClass::__destruct"),
            super::super::symbols::closure_body_symbol("__eir_closure_plan_combo_0"),
            super::super::symbols::closure_wrapper_symbol("__eir_closure_plan_combo_0"),
            super::super::symbols::fcc_wrapper_symbol("plan_fcc"),
        ];
        for symbol in expected_symbols {
            assert!(wat.contains(&format!("${symbol}")), "missing ${symbol}: {wat}");
        }
        let bytes =
            ::wat::parse_str(&wat).unwrap_or_else(|error| panic!("WAT did not assemble: {error}"));
        wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::WASM3)
            .validate_all(&bytes)
            .unwrap_or_else(|error| panic!("WASM did not validate: {error}"));
    }

    /// Verifies an admitted opcode with a malformed shape fails inside validation.
    #[test]
    fn exact_lowering_failure_prevents_capability_acceptance() {
        let mut module = Module::new(Target::wasm());
        let mut main = Function::new("main".to_string(), IrType::Void, PhpType::Void);
        main.flags.is_main = true;
        {
            let mut builder = Builder::new(&mut main);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let _ = builder.emit(
                Op::ArrayNew,
                Vec::new(),
                None,
                IrType::Heap(IrHeapKind::Array),
                PhpType::Array(Box::new(PhpType::Int)),
                Ownership::Owned,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(main);

        let error =
            validate_module(&module).expect_err("malformed lowering must prevent plan acceptance");
        let message = error.to_string();
        assert!(message.contains("array_new without a capacity"), "{message}");
        assert!(
            !message.contains("WASM capability audit found"),
            "the exact lowerer error should be surfaced by the validation boundary: {message}"
        );
    }

    /// Verifies representative late lowerer defects all remain inside validation.
    #[test]
    fn exact_plan_rejects_every_representative_late_unsupported_shape() {
        let cases = [
            (
                invalid_mixed_transfer_module(),
                "unboxing a Mixed cell to Tagged",
            ),
            (invalid_const_str_module(), "unknown string literal"),
            (
                invalid_capture_count_module(),
                "capture_count 1 > params 0",
            ),
            (
                stale_destructor_metadata_module(),
                "class MissingDestructorOwner resolved as __destruct impl does not declare it",
            ),
            (
                invalid_branch_arity_module(),
                "branch arg count 0 != param count 1",
            ),
        ];

        for (module, expected) in cases {
            let error = validate_module(&module)
                .expect_err("the malformed module must not produce an accepted plan");
            let message = error.to_string();
            assert!(message.contains(expected), "missing {expected:?}: {message}");
            assert!(
                !message.contains("WASM capability audit found"),
                "the exact plan boundary should surface {expected:?}: {message}"
            );
        }
    }

    /// Verifies an unsupported FCC wrapper is rejected by the static shape
    /// audit rather than leaking to the exact WAT planning boundary.
    #[test]
    fn rejects_invalid_fcc_wrapper_before_lowering() {
        let error = validate_module(&invalid_fcc_wrapper_module())
            .expect_err("by-reference FCC parameter must fail capability");
        let message = error.to_string();

        assert!(message.contains("WASM capability audit found 1 issue(s)"), "{message}");
        assert!(
            message.contains("callback parameter value is by-reference or variadic"),
            "{message}"
        );
    }

    /// Verifies every `ClosureNew` audits visible wrapper parameters even when a
    /// later local load masks the descriptor from static call resolution.
    #[test]
    fn rejects_masked_closure_wrapper_param_before_lowering() {
        let error = validate_module(&invalid_masked_closure_wrapper_param_module())
            .expect_err("unsupported visible closure parameter must fail capability");
        let message = error.to_string();

        assert!(
            message.contains("callback parameter value has unsupported storage"),
            "{message}"
        );
        assert!(
            message.contains("WASM capability audit found"),
            "wrapper defect must not leak to exact planning: {message}"
        );
    }

    /// Verifies every `ClosureNew` audits its result-boxing contract before the
    /// exact wrapper builder can return a late unsupported error.
    #[test]
    fn rejects_closure_wrapper_return_before_lowering() {
        let error = validate_module(&invalid_closure_wrapper_return_module())
            .expect_err("unsupported closure return must fail capability");
        let message = error.to_string();

        assert!(
            message.contains("callable return TaggedScalar/TaggedScalar cannot be boxed"),
            "{message}"
        );
        assert!(
            message.contains("WASM capability audit found"),
            "wrapper defect must not leak to exact planning: {message}"
        );
    }

    /// Verifies closure-call argument admission mirrors the specialized boxer,
    /// rejecting already-Mixed cells and I64 null sentinels at capability time.
    #[test]
    fn rejects_closure_call_arguments_outside_specialized_boxer() {
        let error = validate_module(&invalid_closure_call_argument_boxing_module())
            .expect_err("unsupported closure arguments must fail capability");
        let message = error.to_string();

        assert!(
            message.contains("WASM capability audit found 2 issue(s)"),
            "{message}"
        );
        assert!(
            message.contains("cannot be boxed from Heap(Mixed)/Mixed"),
            "{message}"
        );
        assert!(message.contains("cannot be boxed from I64/Void"), "{message}");
    }

    /// Verifies aggregate static diagnostics short-circuit exact planning.
    #[test]
    fn aggregate_audit_failure_precedes_exact_planning() {
        let mut module = Module::new(Target::wasm());
        let mut main = Function::new("main".to_string(), IrType::Void, PhpType::Void);
        main.flags.is_main = true;
        {
            let mut builder = Builder::new(&mut main);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let _ = builder.emit(
                Op::StrEq,
                Vec::new(),
                None,
                IrType::I64,
                PhpType::Bool,
                Ownership::NonHeap,
            );
            let _ = builder.emit(
                Op::ArrayNew,
                Vec::new(),
                None,
                IrType::Heap(IrHeapKind::Array),
                PhpType::Array(Box::new(PhpType::Int)),
                Ownership::Owned,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(main);

        let error = validate_module(&module)
            .expect_err("the aggregate audit must reject before exact planning");
        let message = error.to_string();
        assert!(message.contains("WASM capability audit found 1 issue(s)"), "{message}");
        assert!(message.contains("unsupported op str_eq"), "{message}");
        assert!(
            !message.contains("array_new without a capacity"),
            "planning must not mask the stable aggregate diagnostic: {message}"
        );
    }

    /// Verifies all non-emitted EIR function collections are named in one error.
    #[test]
    fn reports_every_non_emitted_function_collection() {
        let mut module = Module::new(Target::wasm());
        module.fiber_wrappers.push(void_function("fiber"));
        module.callback_wrappers.push(void_function("callback"));
        module
            .extern_callback_trampolines
            .push(void_function("extern_callback"));
        module
            .runtime_callable_invokers
            .push(void_function("runtime_invoker"));

        let error = validate_module(&module).expect_err("omitted collections must fail");
        let message = error.to_string();
        assert!(message.contains("fiber_wrappers::fiber"), "{message}");
        assert!(message.contains("callback_wrappers::callback"), "{message}");
        assert!(
            message.contains("extern_callback_trampolines::extern_callback"),
            "{message}"
        );
        assert!(
            message.contains("runtime_callable_invokers::runtime_invoker"),
            "{message}"
        );
        assert!(message.contains("4 issue(s)"), "{message}");
    }

    /// Verifies unsupported opcodes and typed runtime functions are aggregated.
    #[test]
    fn aggregates_instruction_and_runtime_gaps() {
        let mut module = Module::new(Target::wasm());
        let mut function = Function::new("main".to_string(), IrType::Void, PhpType::Void);
        function.flags.is_main = true;
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let _ = builder.emit(
                Op::StrEq,
                Vec::new(),
                None,
                IrType::I64,
                PhpType::Bool,
                Ownership::NonHeap,
            );
            let _ = builder.emit(
                Op::RuntimeCall,
                Vec::new(),
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
                    RuntimeFnId::Count,
                ))),
                IrType::I64,
                PhpType::Int,
                Ownership::NonHeap,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(function);

        let error = validate_module(&module).expect_err("unsupported instructions must fail");
        let message = error.to_string();
        assert!(message.contains("unsupported op str_eq"), "{message}");
        assert!(
            message.contains("unsupported runtime function count"),
            "{message}"
        );
        assert!(message.contains("2 issue(s)"), "{message}");
    }

    /// Verifies runtime functions with known PHP-visible divergences fail together
    /// in the pre-emission audit instead of reaching their partial lowerings.
    #[test]
    fn rejects_semantically_partial_runtime_functions_before_lowering() {
        let mut module = Module::new(Target::wasm());
        let mut function = Function::new("main".to_string(), IrType::Void, PhpType::Void);
        function.flags.is_main = true;
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            for target in [
                RuntimeFnId::ArrayFilter,
                RuntimeFnId::Uasort,
                RuntimeFnId::Uksort,
                RuntimeFnId::ArrayWalk,
                RuntimeFnId::GetClass,
            ] {
                let _ = builder.emit(
                    Op::RuntimeCall,
                    Vec::new(),
                    Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(target))),
                    IrType::Void,
                    PhpType::Void,
                    Ownership::NonHeap,
                );
            }
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(function);

        let error = validate_module(&module)
            .expect_err("known PHP-visible runtime divergences must fail before lowering");
        let message = error.to_string();
        for name in ["array_filter", "uasort", "uksort", "array_walk"] {
            assert!(
                message.contains(&format!("unsupported runtime function {name}")),
                "{message}"
            );
        }
        assert!(
            message.contains("unsupported runtime function get_class shape")
                && message.contains("expected one object operand"),
            "{message}"
        );
        assert!(message.contains("5 issue(s)"), "{message}");
    }

    /// Verifies admitted higher-order runtimes reject unproved callback,
    /// result, and initial-carry shapes during the aggregate capability audit.
    #[test]
    fn rejects_invalid_admitted_runtime_shapes_before_lowering() {
        let mut module = Module::new(Target::wasm());
        let mut function = Function::new("main".to_string(), IrType::Void, PhpType::Void);
        function.flags.is_main = true;
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let callback = builder
                .emit(
                    Op::ConstI64,
                    Vec::new(),
                    Some(Immediate::I64(0)),
                    IrType::I64,
                    PhpType::Callable,
                    Ownership::NonHeap,
                )
                .expect("callback placeholder");
            let array = builder
                .emit(
                    Op::ArrayNew,
                    Vec::new(),
                    Some(Immediate::Capacity(0)),
                    IrType::Heap(IrHeapKind::Array),
                    PhpType::Array(Box::new(PhpType::Int)),
                    Ownership::Owned,
                )
                .expect("array value");
            let _ = builder.emit(
                Op::RuntimeCall,
                vec![callback, array],
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
                    RuntimeFnId::ArrayMap,
                ))),
                IrType::Heap(IrHeapKind::Array),
                PhpType::Array(Box::new(PhpType::Mixed)),
                Ownership::Owned,
            );
            let _ = builder.emit(
                Op::RuntimeCall,
                vec![array, callback],
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
                    RuntimeFnId::Usort,
                ))),
                IrType::F64,
                PhpType::Float,
                Ownership::NonHeap,
            );
            let _ = builder.emit(
                Op::RuntimeCall,
                vec![array, callback, callback],
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
                    RuntimeFnId::ArrayReduce,
                ))),
                IrType::I64,
                PhpType::Int,
                Ownership::NonHeap,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(function);

        let error = validate_module(&module).expect_err("invalid runtime shapes must fail");
        let message = error.to_string();
        assert!(
            message.contains("array_map shape")
                && message.contains("must resolve statically to a direct closure"),
            "{message}"
        );
        assert!(
            message.contains("usort shape") && message.contains("result must be bool/I64"),
            "{message}"
        );
        assert!(
            message.contains("array_reduce shape")
                && message.contains("initial carry must be an int/I64"),
            "{message}"
        );
    }

    /// Verifies the audited P0 instruction forms and main return contract are
    /// rejected by the capability gate rather than by WAT lowering or validation.
    #[test]
    fn rejects_invalid_p0_instruction_shapes_before_lowering() {
        let mut module = Module::new(Target::wasm());
        let method_data = module.data.intern_string("get");
        let mut function = Function::new("main".to_string(), IrType::I64, PhpType::Int);
        function.flags.is_main = true;
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let lhs = builder.emit_const_i64(1);
            let rhs = builder.emit_const_i64(2);
            let _ = builder.emit(
                Op::ICheckedAdd,
                vec![lhs, rhs],
                None,
                IrType::I64,
                PhpType::Int,
                Ownership::NonHeap,
            );
            let _ = builder.emit(
                Op::Cast,
                vec![lhs],
                Some(Immediate::CastTarget(IrType::Str)),
                IrType::Str,
                PhpType::Str,
                Ownership::Owned,
            );
            let array = builder
                .emit(
                    Op::ArrayNew,
                    Vec::new(),
                    Some(Immediate::Capacity(1)),
                    IrType::Heap(IrHeapKind::Array),
                    PhpType::Array(Box::new(PhpType::Int)),
                    Ownership::Owned,
                )
                .expect("array value");
            let _ = builder.emit(
                Op::ArrayGet,
                vec![array, lhs],
                None,
                IrType::F64,
                PhpType::Float,
                Ownership::NonHeap,
            );
            let object = builder
                .emit(
                    Op::ConstNull,
                    Vec::new(),
                    None,
                    IrType::Heap(IrHeapKind::Object),
                    PhpType::Object("P0Shape".to_string()),
                    Ownership::NonHeap,
                )
                .expect("object-shaped placeholder");
            let _ = builder.emit(
                Op::RuntimeCall,
                vec![object],
                Some(Immediate::RuntimeCall(
                    RuntimeCallTarget::ProfiledFunction {
                        target: RuntimeFnId::GetClass,
                        strict_php: true,
                    },
                )),
                IrType::F64,
                PhpType::Float,
                Ownership::NonHeap,
            );
            let mixed = builder
                .emit(
                    Op::MixedBox,
                    vec![lhs],
                    None,
                    IrType::Heap(IrHeapKind::Mixed),
                    PhpType::Mixed,
                    Ownership::Owned,
                )
                .expect("mixed receiver");
            let _ = builder.emit(
                Op::NullsafeMethodCall,
                vec![mixed],
                Some(Immediate::Data(method_data)),
                IrType::I64,
                PhpType::Int,
                Ownership::NonHeap,
            );
            builder.terminate(Terminator::Return { value: Some(lhs) });
        }
        module.add_function(function);

        let error = validate_module(&module).expect_err("P0 shapes must fail at the gate");
        let message = error.to_string();
        for expected in [
            "main must declare a void return",
            "unsupported ichecked_add shape",
            "unsupported cast shape",
            "unsupported array_get shape",
            "unsupported runtime function get_class shape",
            "unsupported nullsafe_method_call shape",
            "main return value would be discarded",
        ] {
            assert!(message.contains(expected), "missing {expected:?}: {message}");
        }
    }

    /// Null-capable array reads are admitted only with the exact Tagged/Mixed
    /// result shapes, including the silent opcode used by null coalescing.
    #[test]
    fn accepts_nullable_array_get_shapes_including_silent_reads() {
        let mut module = Module::new(Target::wasm());
        let mut function = Function::new("reads".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let index = builder.emit_const_i64(0);
            for (op, element, ir_type, php_type, ownership) in [
                (
                    Op::ArrayGet,
                    PhpType::Int,
                    IrType::TaggedScalar,
                    PhpType::TaggedScalar,
                    Ownership::NonHeap,
                ),
                (
                    Op::ArrayGet,
                    PhpType::Bool,
                    IrType::Heap(IrHeapKind::Mixed),
                    PhpType::Mixed,
                    Ownership::Owned,
                ),
                (
                    Op::ArrayGetSilent,
                    PhpType::Str,
                    IrType::Heap(IrHeapKind::Mixed),
                    PhpType::Mixed,
                    Ownership::Owned,
                ),
            ] {
                let array = builder
                    .emit(
                        Op::ArrayNew,
                        Vec::new(),
                        Some(Immediate::Capacity(1)),
                        IrType::Heap(IrHeapKind::Array),
                        PhpType::Array(Box::new(element)),
                        Ownership::Owned,
                    )
                    .expect("array value");
                let _ = builder.emit(
                    op,
                    vec![array, index],
                    None,
                    ir_type,
                    php_type,
                    ownership,
                );
            }
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(function);

        validate_module(&module).expect("nullable array-get shapes must pass the gate");
    }

    /// Legacy non-null result shapes are rejected for both warning and silent
    /// reads so no sentinel, truthy bool, or empty-string alias can escape.
    #[test]
    fn rejects_non_nullable_array_get_result_shapes() {
        let mut module = Module::new(Target::wasm());
        let mut function =
            Function::new("legacy_reads".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let index = builder.emit_const_i64(0);
            for (op, element, ir_type, php_type, ownership) in [
                (
                    Op::ArrayGet,
                    PhpType::Int,
                    IrType::I64,
                    PhpType::Int,
                    Ownership::NonHeap,
                ),
                (
                    Op::ArrayGet,
                    PhpType::Bool,
                    IrType::I64,
                    PhpType::Bool,
                    Ownership::NonHeap,
                ),
                (
                    Op::ArrayGetSilent,
                    PhpType::Str,
                    IrType::Str,
                    PhpType::Str,
                    Ownership::MaybeOwned,
                ),
            ] {
                let array = builder
                    .emit(
                        Op::ArrayNew,
                        Vec::new(),
                        Some(Immediate::Capacity(1)),
                        IrType::Heap(IrHeapKind::Array),
                        PhpType::Array(Box::new(element)),
                        Ownership::Owned,
                    )
                    .expect("array value");
                let _ = builder.emit(
                    op,
                    vec![array, index],
                    None,
                    ir_type,
                    php_type,
                    ownership,
                );
            }
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(function);

        let error =
            validate_module(&module).expect_err("legacy non-null result shapes must fail");
        let message = error.to_string();
        assert_eq!(message.matches("unsupported array_get shape").count(), 2, "{message}");
        assert_eq!(
            message
                .matches("unsupported array_get_silent shape")
                .count(),
            1,
            "{message}"
        );
    }

    /// Indexed-to-hash promotion is admitted with one release-tracked indexed
    /// source and an owned associative result whose integer-keyed value type is
    /// preserved. Capability accepts the shape; hash/CLI tests cover production
    /// lowering and runtime assembly/execution.
    #[test]
    fn accepts_shape_complete_array_to_hash() {
        let mut module = Module::new(Target::wasm());
        let mut function = Function::new("promote".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let array = builder
                .emit(
                    Op::ArrayNew,
                    Vec::new(),
                    Some(Immediate::Capacity(0)),
                    IrType::Heap(IrHeapKind::Array),
                    PhpType::Array(Box::new(PhpType::Int)),
                    Ownership::Owned,
                )
                .expect("array value");
            let _ = builder.emit(
                Op::ArrayToHash,
                vec![array],
                None,
                IrType::Heap(IrHeapKind::Hash),
                PhpType::AssocArray {
                    key: Box::new(PhpType::Int),
                    value: Box::new(PhpType::Int),
                },
                Ownership::Owned,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(function);

        validate_module(&module).expect("shape-complete ArrayToHash must pass planning");
    }

    /// ArrayToHash rejects malformed arity, source type/ownership, result
    /// storage/type compatibility, and result ownership at the static capability
    /// gate instead of falling through to WAT lowering.
    #[test]
    fn rejects_invalid_array_to_hash_shapes_before_lowering() {
        let mut module = Module::new(Target::wasm());
        let mut function =
            Function::new("bad_promotions".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let integer = builder.emit_const_i64(1);
            let array = builder
                .emit(
                    Op::ArrayNew,
                    Vec::new(),
                    Some(Immediate::Capacity(0)),
                    IrType::Heap(IrHeapKind::Array),
                    PhpType::Array(Box::new(PhpType::Int)),
                    Ownership::Owned,
                )
                .expect("array value");
            let borrowed_array = builder
                .emit(
                    Op::ArrayNew,
                    Vec::new(),
                    Some(Immediate::Capacity(0)),
                    IrType::Heap(IrHeapKind::Array),
                    PhpType::Array(Box::new(PhpType::Int)),
                    Ownership::Borrowed,
                )
                .expect("borrowed array value");
            let assoc_int = PhpType::AssocArray {
                key: Box::new(PhpType::Int),
                value: Box::new(PhpType::Int),
            };
            let _ = builder.emit(
                Op::ArrayToHash,
                Vec::new(),
                None,
                IrType::Heap(IrHeapKind::Hash),
                assoc_int.clone(),
                Ownership::Owned,
            );
            let _ = builder.emit(
                Op::ArrayToHash,
                vec![integer],
                None,
                IrType::Heap(IrHeapKind::Hash),
                assoc_int.clone(),
                Ownership::Owned,
            );
            let _ = builder.emit(
                Op::ArrayToHash,
                vec![array],
                None,
                IrType::Heap(IrHeapKind::Array),
                PhpType::Array(Box::new(PhpType::Int)),
                Ownership::Owned,
            );
            let _ = builder.emit(
                Op::ArrayToHash,
                vec![array],
                None,
                IrType::Heap(IrHeapKind::Hash),
                PhpType::AssocArray {
                    key: Box::new(PhpType::Int),
                    value: Box::new(PhpType::Str),
                },
                Ownership::Owned,
            );
            let _ = builder.emit(
                Op::ArrayToHash,
                vec![array],
                None,
                IrType::Heap(IrHeapKind::Hash),
                assoc_int.clone(),
                Ownership::Borrowed,
            );
            let _ = builder.emit(
                Op::ArrayToHash,
                vec![borrowed_array],
                None,
                IrType::Heap(IrHeapKind::Hash),
                assoc_int,
                Ownership::Owned,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(function);

        let error =
            validate_module(&module).expect_err("invalid ArrayToHash shapes must fail");
        let message = error.to_string();
        assert_eq!(
            message.matches("unsupported array_to_hash shape").count(),
            6,
            "{message}"
        );
        for expected in [
            "expected one indexed-array operand",
            "source must be an indexed array",
            "result must be AssocArray<Int, T>",
            "must preserve Int or widen to Mixed",
            "result must own a releasable hash reference",
            "consumed source must own a releasable reference",
        ] {
            assert!(message.contains(expected), "missing {expected:?}: {message}");
        }
    }

    /// Verifies `exit`/`die` in a nested function is rejected until WASM can
    /// unwind and clean every caller-owned frame before `proc_exit`.
    #[test]
    fn rejects_exit_outside_main_before_lowering() {
        let mut module = Module::new(Target::wasm());
        let mut function = Function::new("nested".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let _ = builder.emit(
                Op::LanguageConstructCall,
                Vec::new(),
                None,
                IrType::Void,
                PhpType::Void,
                Ownership::NonHeap,
            );
            builder.terminate(Terminator::Unreachable);
        }
        module.add_function(function);

        let error = validate_module(&module).expect_err("nested exit must fail before lowering");
        let message = error.to_string();
        assert!(
            message.contains("exit/die outside main cannot unwind caller-owned WASM frames"),
            "{message}"
        );
    }

    /// Verifies unsupported block-parameter storage cannot bypass the audit.
    #[test]
    fn audits_block_parameter_types() {
        let mut module = Module::new(Target::wasm());
        let mut function = Function::new("buffer_param".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block(
                "entry",
                vec![(
                    IrType::Heap(crate::ir::IrHeapKind::Buffer),
                    PhpType::Buffer(Box::new(PhpType::Int)),
                )],
            );
            builder.set_entry(entry);
            builder.position_at_end(entry);
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(function);

        let error = validate_module(&module).expect_err("buffer block params must fail");
        let message = error.to_string();
        assert!(message.contains("block#0 param#0"), "{message}");
        assert!(message.contains("unsupported storage type Heap(Buffer)"), "{message}");
    }

    /// Fresh locals and unambiguously owned promoted cells are both admitted as
    /// escaping by-ref closure captures by the pre-emission capability gate.
    #[test]
    fn accepts_fresh_and_owned_by_ref_closure_captures() {
        let mut module = Module::new(Target::wasm());
        let name_id = add_by_ref_int_closure(&mut module, "__cap_owned_ref");

        let mut fresh = Function::new("fresh_creator".to_string(), IrType::Void, PhpType::Void);
        let fresh_slot = fresh.add_local(
            Some("x".to_string()),
            IrType::I64,
            PhpType::Int,
            LocalKind::PhpLocal,
        );
        {
            let mut builder = Builder::new(&mut fresh);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let one = builder.emit_const_i64(1);
            builder.emit_store_local(fresh_slot, one);
            let capture = builder.emit_load_local(fresh_slot, IrType::I64, PhpType::Int);
            let _ = emit_by_ref_closure_new(&mut builder, capture, name_id);
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(fresh);

        let mut owned = Function::new("owned_creator".to_string(), IrType::Void, PhpType::Void);
        let owned_slot = owned.add_local(
            Some("x".to_string()),
            IrType::I64,
            PhpType::Int,
            LocalKind::PhpLocal,
        );
        let owner_slot = owned.add_local(
            Some("__ref_owner_x".to_string()),
            IrType::I64,
            PhpType::Int,
            LocalKind::HiddenTemp,
        );
        {
            let mut builder = Builder::new(&mut owned);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let one = builder.emit_const_i64(1);
            builder.emit_store_local(owned_slot, one);
            let _ = builder.emit(
                Op::PromoteLocalRefCell,
                Vec::new(),
                Some(Immediate::LocalSlotPair {
                    first: owned_slot,
                    second: owner_slot,
                }),
                IrType::Void,
                PhpType::Int,
                Ownership::NonHeap,
            );
            let capture = builder
                .emit(
                    Op::LoadRefCell,
                    Vec::new(),
                    Some(Immediate::LocalSlot(owned_slot)),
                    IrType::I64,
                    PhpType::Int,
                    Ownership::NonHeap,
                )
                .expect("owned capture");
            let _ = emit_by_ref_closure_new(&mut builder, capture, name_id);
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(owned);

        validate_module(&module).expect("owned by-ref capture shapes must be admitted");
    }

    /// By-ref parameters and foreach element addresses are borrowed bindings;
    /// both must be rejected before WAT emission when a closure would outlive them.
    #[test]
    fn rejects_borrowed_by_ref_closure_captures_before_emission() {
        let mut module = Module::new(Target::wasm());
        let name_id = add_by_ref_int_closure(&mut module, "__cap_borrowed_ref");

        let mut parameter =
            Function::new("parameter_creator".to_string(), IrType::Void, PhpType::Void);
        parameter.params.push(FunctionParam {
            name: "x".to_string(),
            ir_type: IrType::I64,
            php_type: PhpType::Int,
            by_ref: true,
            variadic: false,
        });
        let parameter_slot = parameter.add_local(
            Some("x".to_string()),
            IrType::I64,
            PhpType::Int,
            LocalKind::PhpLocal,
        );
        {
            let mut builder = Builder::new(&mut parameter);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let capture = builder
                .emit(
                    Op::LoadRefCell,
                    Vec::new(),
                    Some(Immediate::LocalSlot(parameter_slot)),
                    IrType::I64,
                    PhpType::Int,
                    Ownership::NonHeap,
                )
                .expect("borrowed parameter capture");
            let _ = emit_by_ref_closure_new(&mut builder, capture, name_id);
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(parameter);

        let mut interior =
            Function::new("interior_creator".to_string(), IrType::Void, PhpType::Void);
        let interior_slot = interior.add_local(
            Some("item".to_string()),
            IrType::I64,
            PhpType::Int,
            LocalKind::PhpLocal,
        );
        {
            let mut builder = Builder::new(&mut interior);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let _ = builder.emit(
                Op::IterCurrentValueRef,
                Vec::new(),
                Some(Immediate::LocalSlot(interior_slot)),
                IrType::Void,
                PhpType::Int,
                Ownership::NonHeap,
            );
            let capture = builder
                .emit(
                    Op::LoadRefCell,
                    Vec::new(),
                    Some(Immediate::LocalSlot(interior_slot)),
                    IrType::I64,
                    PhpType::Int,
                    Ownership::NonHeap,
                )
                .expect("borrowed interior capture");
            let _ = emit_by_ref_closure_new(&mut builder, capture, name_id);
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(interior);

        let error = validate_module(&module).expect_err("borrowed bindings must not escape");
        let message = error.to_string();
        assert!(message.contains("parameter_creator"), "{message}");
        assert!(message.contains("interior_creator"), "{message}");
        assert_eq!(
            message
                .matches("cannot escape non-owned ref-bound local#0")
                .count(),
            2,
            "{message}"
        );
    }
}
