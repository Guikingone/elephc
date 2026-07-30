//! Purpose:
//! Per-identity disposition classification for the WASM capability inventory.
//!
//! Called from:
//! - `super::build_report` to turn every enumerated EIR identity into one row
//!   with exactly one `Disposition`.
//!
//! Key details:
//! - Supported/missing reuses the exhaustive `codegen_wasm::capability`
//!   classifiers; `excluded` is reserved for native-only Elephc extensions
//!   (ptr, buffer, packed, native `extern`, native bridge/system-library
//!   requirements, and the web SAPI) with a stable contract and matching CLI
//!   diagnostic. Ordinary PHP without a WASM lowerer is `missing`, never
//!   silently `excluded`.
#![allow(dead_code)]

use super::evidence::op_supported_evidence;
use super::schema::{
    Disposition, ExecutionMode, Exclusion, InventoryRow, ShapePredicate, SupportedEvidence,
};
use crate::codegen_wasm::capability::{
    runtime_function_is_supported, terminator_is_supported, terminator_name,
    unary_string_name,
};
use crate::ir::{Op, RuntimeFnId, Terminator, UnaryStringRuntime};

/// Returns the exclusion contract for a native-only `Op` variant, if any.
pub(super) fn op_exclusion(op: Op) -> Option<Exclusion> {
    match op {
        Op::PtrCast
        | Op::PtrRead
        | Op::PtrWrite
        | Op::PtrReadString
        | Op::PtrWriteString
        | Op::PtrOffset
        | Op::PtrCheckNonnull => Some(Exclusion {
            category: "native-ffi-ptr",
            reason: "elephc-only raw pointer extension; not PHP-visible",
            owner: "wasm-backend",
            removal_gate: "a WASM linear-memory pointer ABI with bounds-checked lowering",
            diagnostic: "capability audit rejects `ptr` ops as unsupported on wasm32-wasi",
        }),
        Op::BufferNew
        | Op::BufferLen
        | Op::BufferGet
        | Op::BufferSet
        | Op::BufferFree => Some(Exclusion {
            category: "native-buffer",
            reason: "elephc-only `buffer<T>` extension; not PHP-visible",
            owner: "wasm-backend",
            removal_gate: "a WASM buffer lowering over linear memory",
            diagnostic: "capability audit rejects `buffer` ops as unsupported on wasm32-wasi",
        }),
        Op::PackedFieldGet | Op::PackedFieldSet => Some(Exclusion {
            category: "native-packed",
            reason: "elephc-only `packed class` extension; not PHP-visible",
            owner: "wasm-backend",
            removal_gate: "a WASM packed-field storage lowering",
            diagnostic: "capability audit rejects `packed` ops as unsupported on wasm32-wasi",
        }),
        Op::ExternCall | Op::ExternGlobalLoad | Op::ExternGlobalStore => Some(Exclusion {
            category: "native-extern",
            reason: "native `extern` FFI requiring host linker libraries",
            owner: "wasm-backend",
            removal_gate: "a WASM component-model import surface and prelude rewriting",
            diagnostic: "--link, --link-path, and --framework are not supported for --target wasm32-wasi",
        }),
        _ => None,
    }
}


/// Returns the exclusion contract for an Elephc-only runtime identity, if any.
pub(super) fn runtime_fn_exclusion(id: RuntimeFnId) -> Option<Exclusion> {
    if matches!(id, RuntimeFnId::Header | RuntimeFnId::HttpResponseCode) {
        return Some(Exclusion {
            category: "web-sapi",
            reason: "HTTP/web SAPI builtin requiring the --web server entry point",
            owner: "wasm-backend",
            removal_gate: "a WASI HTTP/component-model server surface",
            diagnostic: "--web is not yet supported for --target wasm32-wasi",
        });
    }
    if matches!(
        id,
        RuntimeFnId::ElephcPtrIsNull
            | RuntimeFnId::ElephcPtrReadString
            | RuntimeFnId::ElephcPtrWriteString
            | RuntimeFnId::BufferFree
            | RuntimeFnId::BufferLen
            | RuntimeFnId::Ptr
            | RuntimeFnId::PtrGet
            | RuntimeFnId::PtrIsNull
            | RuntimeFnId::PtrNull
            | RuntimeFnId::PtrOffset
            | RuntimeFnId::PtrRead8
            | RuntimeFnId::PtrRead16
            | RuntimeFnId::PtrRead32
            | RuntimeFnId::PtrReadString
            | RuntimeFnId::PtrSet
            | RuntimeFnId::PtrSizeof
            | RuntimeFnId::PtrWrite8
            | RuntimeFnId::PtrWrite16
            | RuntimeFnId::PtrWrite32
            | RuntimeFnId::PtrWriteString
            | RuntimeFnId::ZvalFree
            | RuntimeFnId::ZvalPack
            | RuntimeFnId::ZvalType
            | RuntimeFnId::ZvalUnpack
            | RuntimeFnId::ClassAttributeArgs
            | RuntimeFnId::ClassAttributeNames
            | RuntimeFnId::ClassGetAttributes
    ) {
        return Some(Exclusion {
            category: "elephc-native-extension",
            reason: "Elephc-only native pointer/buffer/zval/attribute extension; not PHP-visible",
            owner: "wasm-backend",
            removal_gate: "an explicit WASM extension ABI with bounds-checked linear-memory semantics",
            diagnostic: "unsupported runtime function <eir-name>",
        });
    }
    None
}


/// Returns the evidence record for a supported `RuntimeFnId`, if it is supported.
pub(super) fn runtime_fn_supported_evidence(id: RuntimeFnId) -> Option<SupportedEvidence> {
    if !runtime_function_is_supported(id) {
        return None;
    }
    let (lowerer, tests) = match id {
        RuntimeFnId::GetClass => (
            "codegen_wasm::classes::lower_get_class",
            &["codegen_wasm::tests::get_class_object_returns_class_name"][..],
        ),
        RuntimeFnId::ArrayMap => (
            "codegen_wasm::inst::lower_array_map",
            &["codegen_wasm::closures::tests::array_map_lowering_via_builtin_call_returns_4220"]
                [..],
        ),
        RuntimeFnId::Usort => (
            "codegen_wasm::inst::lower_user_sort(usort)",
            &["codegen_wasm::closures::tests::usort_lowering_writes_back_to_local"][..],
        ),
        RuntimeFnId::ArrayReduce => (
            "codegen_wasm::inst::lower_array_reduce",
            &["codegen_wasm::closures::tests::array_reduce_lowering_boxes_mixed_result"][..],
        ),
        _ => return None,
    };
    Some(SupportedEvidence {
        backend: "codegen_wasm::runtime",
        lowerer,
        producers: &["PHP-visible builtin whose runtime target is lowered on WASM"],
        tests,
    })
}


/// Classifies one `Op` into an inventory row with exactly one disposition.
pub(super) fn op_row(op: Op) -> InventoryRow {
    let name = op.name().to_string();
    if let Some(exclusion) = op_exclusion(op) {
        return InventoryRow {
            name,
            family: "op",
            enum_name: "Op",
            disposition: Disposition::Excluded,
            supported: None,
            excluded: Some(exclusion),
            missing: None,
        };
    }
    if let Some(evidence) = op_supported_evidence(op) {
        return InventoryRow {
            name,
            family: "op",
            enum_name: "Op",
            disposition: Disposition::Supported,
            supported: Some(evidence),
            excluded: None,
            missing: None,
        };
    }
    InventoryRow {
        name,
        family: "op",
        enum_name: "Op",
        disposition: Disposition::Missing,
        supported: None,
        excluded: None,
        missing: Some("ordinary PHP reachable from the public frontend; WASM lowerer absent"),
    }
}


/// Classifies one `RuntimeFnId` into an inventory row with exactly one disposition.
pub(super) fn runtime_fn_row(id: RuntimeFnId) -> InventoryRow {
    let name = id.as_eir().to_string();
    if let Some(exclusion) = runtime_fn_exclusion(id) {
        return InventoryRow {
            name,
            family: "runtime_fn",
            enum_name: "RuntimeFnId",
            disposition: Disposition::Excluded,
            supported: None,
            excluded: Some(exclusion),
            missing: None,
        };
    }
    if let Some(evidence) = runtime_fn_supported_evidence(id) {
        return InventoryRow {
            name,
            family: "runtime_fn",
            enum_name: "RuntimeFnId",
            disposition: Disposition::Supported,
            supported: Some(evidence),
            excluded: None,
            missing: None,
        };
    }
    InventoryRow {
        name,
        family: "runtime_fn",
        enum_name: "RuntimeFnId",
        disposition: Disposition::Missing,
        supported: None,
        excluded: None,
        missing: Some("ordinary PHP builtin reachable from the public frontend; WASM lowerer absent"),
    }
}


/// Classifies one `UnaryStringRuntime` into an inventory row with exactly one disposition.
pub(super) fn unary_string_row(target: UnaryStringRuntime) -> InventoryRow {
    InventoryRow {
        name: unary_string_name(target).to_string(),
        family: "unary_string",
        enum_name: "UnaryStringRuntime",
        disposition: Disposition::Missing,
        supported: None,
        excluded: None,
        missing: Some(
            "ordinary PHP string transform reachable from the public frontend; WASM lowerer absent",
        ),
    }
}


/// Classifies one terminator kind into an inventory row with exactly one disposition.
pub(super) fn terminator_row(terminator: &Terminator) -> InventoryRow {
    let name = terminator_name(terminator).to_string();
    if terminator_is_supported(terminator) {
        InventoryRow {
            name,
            family: "terminator",
            enum_name: "Terminator",
            disposition: Disposition::Supported,
            supported: Some(SupportedEvidence {
                backend: "codegen_wasm::function",
                lowerer: "lower_terminator",
                producers: &["branches", "switch", "return", "unreachable CFG proof"],
                tests: &[
                    "codegen_wasm::tests::main_condbr_lowers_to_valid_wasm",
                    "codegen_wasm::tests::br_with_args_lowers_to_valid_wasm",
                    "codegen_wasm::tests::switch_lowers_to_valid_wasm",
                    "codegen_wasm::tests::main_command_has_a_complete_unreachable_inventory",
                ],
            }),
            excluded: None,
            missing: None,
        }
    } else {
        InventoryRow {
            name,
            family: "terminator",
            enum_name: "Terminator",
            disposition: Disposition::Missing,
            supported: None,
            excluded: None,
            missing: Some("ordinary PHP control-flow terminator; WASM lowerer absent"),
        }
    }
}


/// Classifies the four `RuntimeCallTarget` forms into inventory rows.
pub(super) fn runtime_call_target_rows() -> Vec<InventoryRow> {
    let form = |name: &'static str,
                disposition: Disposition,
                supported: Option<SupportedEvidence>,
                missing: Option<&'static str>| InventoryRow {
        name: name.to_string(),
        family: "runtime_call_target",
        enum_name: "RuntimeCallTarget",
        disposition,
        supported,
        excluded: None,
        missing,
    };
    vec![
        form(
            "array.fetch_for_write",
            Disposition::Missing,
            None,
            Some(
                "ordinary PHP nested-array write helper; WASM lowerer absent",
            ),
        ),
        form(
            "unary_string",
            Disposition::Missing,
            None,
            Some(
                "ordinary PHP string transform dispatch form; WASM lowerer absent (see unary_string family)",
            ),
        ),
        form(
            "function",
            Disposition::Supported,
            Some(SupportedEvidence {
                backend: "codegen_wasm::runtime",
                lowerer: "check_runtime_call",
                producers: &["typed runtime call dispatch"],
                tests: &[
                    "codegen_wasm::closures::tests::array_map_lowering_via_builtin_call_returns_4220",
                ],
            }),
            None,
        ),
        form(
            "profiled_function",
            Disposition::Supported,
            Some(SupportedEvidence {
                backend: "codegen_wasm::runtime",
                lowerer: "check_runtime_call",
                producers: &["strict-PHP-profiled typed runtime call dispatch"],
                tests: &["codegen_wasm::tests::get_class_object_returns_class_name"],
            }),
            None,
        ),
    ]
}


/// Returns the shape predicates enforced before WAT staging.
pub(super) fn shape_predicates() -> Vec<ShapePredicate> {
    [
        "terminator_transfer_shape_issue",
        "method_call_on_null_error_shape_issue",
        "array_offset_on_null_warning_shape_issue",
        "unset_owned_temp_shape_issue",
        "first_class_callable_new_shape_issue",
        "checked_int_binop_shape_issue",
        "value_transfer_shape_issue",
        "local_transfer_shape_issue",
        "load_global_shape_issue",
        "store_ref_cell_shape_issue",
        "forward_transfer_shape_issue",
        "cast_shape_issue",
        "truthiness_shape_issue",
        "array_store_shape_issue",
        "iter_start_shape_issue",
        "iter_current_value_ref_shape_issue",
        "array_get_shape_issue",
        "hash_get_shape_issue",
        "hash_key_diagnostic_issue",
        "hash_store_value_diagnostic_issue",
        "array_to_hash_shape_issue",
        "direct_call_shape_issue",
        "by_ref_source_shape_issue",
        "method_call_shape_issue",
        "object_new_shape_issue",
        "property_get_shape_issue",
        "property_set_shape_issue",
        "static_method_call_shape_issue",
        "method_signature_shape_issue",
        "method_body_signature_shape_issue",
        "method_body_argument_shape_issue",
        "direct_method_result_shape_issue",
        "mixed_method_issue",
        "runtime_function_shape_issue",
        "get_class_shape_issue",
        "array_map_shape_issue",
        "usort_shape_issue",
        "array_reduce_shape_issue",
        "closure_call_shape_issue",
        "closure_new_by_ref_capture_issue",
        "callable_argument_contract_issue",
        "callable_result_contract_issue",
        "callable_wrapper_issue",
        "callable_wrapper_signature_issue",
        "callable_descriptor_invoke_shape_issue",
        "closure_result_shape_issue",
        "iterator_alias_mutation_issue",
    ]
    .into_iter()
    .map(|name| ShapePredicate {
        name,
        disposition: "enforced",
    })
    .collect()
}


/// Returns the public execution modes that can reach the WASM backend.
pub(super) fn execution_modes() -> Vec<ExecutionMode> {
    vec![
        ExecutionMode {
            mode: "command",
            reachable: true,
        },
        ExecutionMode {
            mode: "npm",
            reachable: true,
        },
    ]
}


/// Returns one representative `Terminator` per variant kind for enumeration.
pub(super) fn terminator_representatives() -> Vec<Terminator> {
    use crate::ir::{BlockId, DataId, ValueId};
    let dummy_block = BlockId::from_raw(0);
    let dummy_value = ValueId::from_raw(0);
    let dummy_data = DataId::from_raw(0);
    vec![
        Terminator::Br {
            target: dummy_block,
            args: Vec::new(),
        },
        Terminator::CondBr {
            cond: dummy_value,
            then_target: dummy_block,
            then_args: Vec::new(),
            else_target: dummy_block,
            else_args: Vec::new(),
        },
        Terminator::Switch {
            scrutinee: dummy_value,
            cases: Vec::new(),
            default: dummy_block,
            default_args: Vec::new(),
        },
        Terminator::Return { value: None },
        Terminator::Throw { value: dummy_value },
        Terminator::Fatal { message: dummy_data },
        Terminator::GeneratorSuspend {
            key: None,
            value: None,
            resume: dummy_block,
            resume_args: Vec::new(),
        },
        Terminator::Unreachable,
    ]
}
