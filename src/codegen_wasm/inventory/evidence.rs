//! Purpose:
//! Producer/test evidence tables for supported `Op` variants in the WASM
//! capability inventory.
//!
//! Called from:
//! - `super::classify::op_row` to attach backend lowerer and test evidence to
//!   each supported `Op` row.
//!
//! Key details:
//! - Evidence is grouped by lowering family so the supported `Op` variants
//!   share compact, maintainable records instead of one row per producer.
#![allow(dead_code)]

use super::schema::SupportedEvidence;
use crate::codegen_wasm::capability::op_is_supported;
use crate::ir::Op;

/// Returns the evidence record for a supported `Op` variant, if it is supported.
pub(super) fn op_supported_evidence(op: Op) -> Option<SupportedEvidence> {
    if !op_is_supported(op) {
        return None;
    }
    let group = op_evidence_group(op);
    Some(supported_evidence_for_group(group))
}


/// Maps a supported `Op` to its shared evidence-group key.
pub(super) fn op_evidence_group(op: Op) -> &'static str {
    match op {
        Op::ConstI64 | Op::ConstF64 | Op::ConstStr | Op::ConstNull | Op::ConstBool => "const",
        Op::LoadLocal | Op::StoreLocal | Op::UnsetLocal => "transfer_local",
        Op::LoadRefCell
        | Op::StoreRefCell
        | Op::PromoteLocalRefCell
        | Op::AliasLocalRefCell
        | Op::ReleaseLocalRefCell => "transfer_refcell",
        Op::LoadGlobal => "transfer_global_load",
        Op::IAdd
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
        | Op::IShrA => "scalar_int",
        Op::FAdd | Op::FSub | Op::FMul | Op::FDiv | Op::FNeg => "float",
        Op::ICmp | Op::FCmp | Op::IsNull | Op::IsTruthy | Op::InstanceOf => "compare",
        Op::IToF | Op::FToI | Op::Cast => "cast",
        Op::MixedBox
        | Op::MixedTagOf
        | Op::MixedUnbox
        | Op::ArrayToMixed
        | Op::HashToMixed => "mixed",
        Op::StrConcat | Op::ConcatReset | Op::StrLen => "string",
        Op::ArrayNew
        | Op::ArrayLen
        | Op::ArrayGet
        | Op::ArrayGetSilent
        | Op::ArraySet
        | Op::ArrayPush
        | Op::ArrayUnion
        | Op::ArrayIsset
        | Op::ArrayToHash => "array_indexed",
        Op::HashNew
        | Op::HashGet
        | Op::HashGetSilent
        | Op::HashSet
        | Op::HashUnset
        | Op::HashAppend
        | Op::HashUnion
        | Op::ArrayHashUnion
        | Op::HashArrayUnion
        | Op::HashLen
        | Op::HashIsset => "hash",
        Op::IterStart
        | Op::IterCurrentKey
        | Op::IterCurrentValue
        | Op::IterCurrentValueRef
        | Op::IterNext
        | Op::IterEnd => "iter",
        Op::ObjectNew | Op::PropGet | Op::PropSet | Op::NullsafePropGet => "object",
        Op::MethodCall | Op::NullsafeMethodCall | Op::StaticMethodCall => "method",
        Op::Call | Op::LanguageConstructCall | Op::RuntimeCall => "call",
        Op::ClosureNew | Op::ClosureCapture | Op::ClosureCall
        | Op::FirstClassCallableNew
        | Op::CallableDescriptorInvoke => "closure",
        Op::EchoValue | Op::PrintValue | Op::WriteStrStdout => "echo",
        Op::Warn => "warn",
        Op::ThrowError => "throw_error",
        Op::Acquire | Op::Release | Op::Move | Op::Borrow | Op::Nop => "ownership",
        _ => "other",
    }
}


/// Returns the shared evidence record for a supported-group key.
pub(super) fn supported_evidence_for_group(group: &'static str) -> SupportedEvidence {
    match group {
        "const" => SupportedEvidence {
            backend: "codegen_wasm::inst",
            lowerer: "lower_const",
            producers: &["integer/float/string/null/bool literals"],
            tests: &["codegen_wasm::tests::echo_integers_writes_to_stdout"],
        },
        "transfer_local" => SupportedEvidence {
            backend: "codegen_wasm::inst",
            lowerer: "lower_load_local/store_local",
            producers: &["local variable load/store", "unset"],
            tests: &["codegen_wasm::tests::echo_integers_writes_to_stdout"],
        },
        "transfer_refcell" => SupportedEvidence {
            backend: "codegen_wasm::refcell",
            lowerer: "refcell lowerers",
            producers: &["by-reference captures", "refcell load/store/promote"],
            tests: &[
                "codegen_wasm::tests::ref_cell_promotion_is_runtime_idempotent_across_branches",
                "codegen_wasm::tests::acquired_ref_cell_return_survives_owner_epilogue",
            ],
        },
        "transfer_global_load" => SupportedEvidence {
            backend: "codegen_wasm::inst",
            lowerer: "lower_load_global",
            producers: &["superglobal reads in command mode"],
            tests: &["codegen_wasm::tests::argc_reports_argument_count"],
        },
        "scalar_int" => SupportedEvidence {
            backend: "codegen_wasm::inst",
            lowerer: "lower_int_binop",
            producers: &["integer arithmetic and bitwise operators"],
            tests: &["codegen_wasm::tests::echo_integers_writes_to_stdout"],
        },
        "float" => SupportedEvidence {
            backend: "codegen_wasm::float",
            lowerer: "lower_float_binop",
            producers: &["float arithmetic"],
            tests: &[
                "codegen_wasm::tests::echo_float_writes_to_stdout",
                "codegen_wasm::tests::echo_mixed_float_writes_to_stdout",
            ],
        },
        "compare" => SupportedEvidence {
            backend: "codegen_wasm::inst",
            lowerer: "lower_compare",
            producers: &["comparison and truthiness operators", "instanceof"],
            tests: &["codegen_wasm::tests::echo_integers_writes_to_stdout"],
        },
        "cast" => SupportedEvidence {
            backend: "codegen_wasm::inst",
            lowerer: "lower_cast",
            producers: &["explicit and implicit numeric casts"],
            tests: &["codegen_wasm::tests::echo_float_writes_to_stdout"],
        },
        "mixed" => SupportedEvidence {
            backend: "codegen_wasm::mixed",
            lowerer: "mixed box/unbox/tag",
            producers: &["Mixed cell construction and extraction"],
            tests: &["codegen_wasm::tests::echo_mixed_float_writes_to_stdout"],
        },
        "string" => SupportedEvidence {
            backend: "codegen_wasm::inst",
            lowerer: "lower_str_concat/len",
            producers: &["string concatenation", "strlen"],
            tests: &[
                "codegen_wasm::tests::chained_concat_echoes_correctly",
                "codegen_wasm::tests::strlen_of_literal_invokes_correctly",
            ],
        },
        "array_indexed" => SupportedEvidence {
            backend: "codegen_wasm::arrays",
            lowerer: "lower_array_*",
            producers: &["indexed array literals", "indexed reads/writes", "isset", "append"],
            tests: &["codegen_wasm::tests::echo_integers_writes_to_stdout"],
        },
        "hash" => SupportedEvidence {
            backend: "codegen_wasm::hashes",
            lowerer: "lower_hash_*",
            producers: &["associative array literals", "hash reads/writes/isset/unset/append"],
            tests: &[
                "codegen_wasm::tests::hash_set_mixed_int_cast_fails_closed",
                "codegen_wasm::tests::hash_set_mixed_float_cast_fails_closed",
                "codegen_wasm::tests::hash_set_mixed_string_cast_fails_closed",
            ],
        },
        "iter" => SupportedEvidence {
            backend: "codegen_wasm::inst",
            lowerer: "lower_iter_*",
            producers: &["foreach over concrete arrays/hashes"],
            tests: &["codegen_wasm::tests::echo_integers_writes_to_stdout"],
        },
        "object" => SupportedEvidence {
            backend: "codegen_wasm::objects",
            lowerer: "lower_object_new/prop_get/prop_set",
            producers: &["object construction", "property reads/writes"],
            tests: &["codegen_wasm::tests::echo_integers_writes_to_stdout"],
        },
        "method" => SupportedEvidence {
            backend: "codegen_wasm::methods",
            lowerer: "lower_method_call",
            producers: &["method calls", "nullsafe method calls", "static method calls"],
            tests: &[
                "codegen_wasm::capability::tests::method_capability_rejects_non_public_candidates",
            ],
        },
        "call" => SupportedEvidence {
            backend: "codegen_wasm::calls",
            lowerer: "lower_call/runtime_call",
            producers: &["function calls", "language-construct calls", "runtime calls"],
            tests: &[
                "codegen_wasm::capability::tests::direct_call_shape_rejects_arity_mismatch_before_lowering",
            ],
        },
        "closure" => SupportedEvidence {
            backend: "codegen_wasm::closures",
            lowerer: "lower_closure_*",
            producers: &["closure creation/capture/call", "first-class callable", "callable invoke"],
            tests: &["codegen_wasm::tests::echo_integers_writes_to_stdout"],
        },
        "echo" => SupportedEvidence {
            backend: "codegen_wasm::inst",
            lowerer: "lower_echo",
            producers: &["echo", "print"],
            tests: &[
                "codegen_wasm::tests::echo_integers_writes_to_stdout",
                "codegen_wasm::tests::echo_string_literal_writes_to_stdout",
                "codegen_wasm::tests::echo_booleans_writes_to_stdout",
            ],
        },
        "warn" => SupportedEvidence {
            backend: "codegen_wasm::inst",
            lowerer: "lower_warn (array-offset-on-null)",
            producers: &["array offset read through null"],
            tests: &["codegen_wasm::capability::tests"],
        },
        "throw_error" => SupportedEvidence {
            backend: "codegen_wasm::inst",
            lowerer: "lower_throw_error (method-on-null boundary)",
            producers: &["method call on null in a command module"],
            tests: &[
                "codegen_wasm::capability::tests::rejects_method_on_null_error_without_command_runtime",
            ],
        },
        "ownership" => SupportedEvidence {
            backend: "codegen_wasm::inst",
            lowerer: "lower_move/borrow/acquire/release/nop",
            producers: &["ownership transfers", "no-op"],
            tests: &["codegen_wasm::tests::exit_runs_owned_local_destructors_before_terminating"],
        },
        _ => SupportedEvidence {
            backend: "codegen_wasm::inst",
            lowerer: "lower_op",
            producers: &["mixed EIR lowering"],
            tests: &["codegen_wasm::tests"],
        },
    }
}

