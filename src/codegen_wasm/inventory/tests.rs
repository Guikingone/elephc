//! Purpose:
//! Schema and disposition regression tests for the WASM capability inventory.
//!
//! Called from:
//! - `cargo test` through the `inventory` module's `#[cfg(test)]` harness.
//!
//! Key details:
//! - Every identity must carry exactly one disposition; totals are derived
//!   from the enumeration; missing reachable identities fail the gate;
//!   excluded rows carry a complete contract and matching diagnostic.
#![allow(dead_code)]

use super::*;
use crate::codegen_wasm::capability::{op_is_supported, runtime_function_is_supported};
use std::collections::HashSet;

/// Extracts top-level variant names from a repo-owned enum declaration.
///
/// The parser is deliberately narrow: the audited enums use Rust identifiers
/// and optional tuple/struct payloads. It is a CI drift tripwire, not a general
/// Rust parser.
fn declared_enum_variants(source: &str, declaration: &str) -> Vec<String> {
    let declaration_start = source
        .find(declaration)
        .unwrap_or_else(|| panic!("missing enum declaration {declaration:?}"));
    let body_start = source[declaration_start..]
        .find('{')
        .map(|offset| declaration_start + offset + 1)
        .expect("enum declaration has a body");
    let mut variants = Vec::new();
    let mut payload_depth = 0usize;
    for raw_line in source[body_start..].lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("///") || line.starts_with("#[") {
            continue;
        }
        if payload_depth == 0 && line.starts_with('}') {
            break;
        }
        if payload_depth == 0 {
            let name = line
                .split(|character: char| {
                    matches!(character, ',' | '(' | '{' | '=') || character.is_whitespace()
                })
                .next()
                .unwrap_or_default();
            if name
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_uppercase())
            {
                variants.push(name.to_string());
            }
        }
        if !line.starts_with("//") {
            payload_depth += line.bytes().filter(|byte| *byte == b'{').count();
            payload_depth -= line.bytes().filter(|byte| *byte == b'}').count();
        }
    }
    variants
}

/// Extracts every capability predicate whose function name ends in `_issue`.
fn declared_shape_predicates(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in source.lines().map(str::trim_start) {
        let Some(fn_offset) = line.find("fn ") else {
            continue;
        };
        let tail = &line[fn_offset + 3..];
        let name = tail
            .split(|character: char| character == '(' || character.is_whitespace())
            .next()
            .unwrap_or_default();
        if name.ends_with("_issue") {
            names.push(name.to_string());
        }
    }
    names.sort();
    names.dedup();
    names
}

/// Verifies every enumerated identity carries exactly one disposition and
/// the report validates against the W0 schema.
#[test]
fn every_identity_has_exactly_one_disposition() {
    let report = build_report();
    let errors = validate_report(&report);
    assert!(errors.is_empty(), "schema/disposition errors:\n{}", errors.join("\n"));
    for family in report.families.values() {
        for row in &family.rows {
            let payloads = row.supported.is_some() as usize
                + row.excluded.is_some() as usize
                + row.missing.is_some() as usize;
            assert_eq!(
                payloads, 1,
                "row {:?} ({:?}) carries {} payload fields",
                row.name, row.disposition, payloads
            );
        }
    }
}

/// Verifies the canonical enumerators and the exhaustive capability
/// classifiers agree on the supported/missing split, so the report cannot
/// drift from `codegen_wasm::capability`.
#[test]
fn inventory_matches_capability_classifiers() {
    for op in Op::all() {
        let row = op_row(*op);
        if op_exclusion(*op).is_some() {
            assert_eq!(row.disposition, Disposition::Excluded, "op {:?}", op.name());
        } else if op_is_supported(*op) {
            assert_eq!(row.disposition, Disposition::Supported, "op {:?}", op.name());
        } else {
            assert_eq!(row.disposition, Disposition::Missing, "op {:?}", op.name());
        }
    }
    for id in RuntimeFnId::all() {
        let row = runtime_fn_row(*id);
        if runtime_fn_exclusion(*id).is_some() {
            assert_eq!(row.disposition, Disposition::Excluded, "runtime_fn {:?}", id.as_eir());
        } else if runtime_function_is_supported(*id) {
            assert_eq!(row.disposition, Disposition::Supported, "runtime_fn {:?}", id.as_eir());
        } else {
            assert_eq!(row.disposition, Disposition::Missing, "runtime_fn {:?}", id.as_eir());
        }
    }
}

/// Verifies no `Op::all()` variant is duplicated and the enumeration covers
/// every variant the exhaustive `Op::name` classifier knows.
#[test]
fn op_enumeration_has_no_duplicates() {
    let mut seen = HashSet::new();
    for op in Op::all() {
        assert!(seen.insert(op.name()), "duplicate Op name {:?}", op.name());
    }
    let mut rt_seen = HashSet::new();
    for id in RuntimeFnId::all() {
        assert!(rt_seen.insert(id.as_eir()), "duplicate RuntimeFnId name {:?}", id.as_eir());
    }
    let mut un_seen = HashSet::new();
    for u in UnaryStringRuntime::all() {
        assert!(un_seen.insert(u.as_eir()), "duplicate UnaryStringRuntime name {:?}", u.as_eir());
    }
}

/// Verifies the three `::all()` enumerators exactly match their enum declarations.
#[test]
fn enum_enumerators_match_source_declarations() {
    let op_declared =
        declared_enum_variants(include_str!("../../ir/instr.rs"), "pub enum Op");
    let op_enumerated: Vec<String> = Op::all().iter().map(|op| format!("{op:?}")).collect();
    assert_eq!(op_enumerated, op_declared, "Op::all() drifted");

    let runtime_declared = declared_enum_variants(
        include_str!("../../ir/runtime_fn.rs"),
        "pub enum RuntimeFnId",
    );
    let runtime_enumerated: Vec<String> = RuntimeFnId::all()
        .iter()
        .map(|id| format!("{id:?}"))
        .collect();
    assert_eq!(
        runtime_enumerated, runtime_declared,
        "RuntimeFnId::all() drifted"
    );

    let unary_declared = declared_enum_variants(
        include_str!("../../ir/runtime_call.rs"),
        "pub enum UnaryStringRuntime",
    );
    let unary_enumerated: Vec<String> = UnaryStringRuntime::all()
        .iter()
        .map(|target| format!("{target:?}"))
        .collect();
    assert_eq!(
        unary_enumerated, unary_declared,
        "UnaryStringRuntime::all() drifted"
    );
}

/// Verifies payload enum forms and capability predicates cannot outgrow the inventory.
#[test]
fn payload_forms_and_shape_predicates_match_source_declarations() {
    let runtime_targets = declared_enum_variants(
        include_str!("../../ir/runtime_call.rs"),
        "pub enum RuntimeCallTarget",
    );
    assert_eq!(
        runtime_targets,
        ["ArrayFetchForWrite", "UnaryString", "Function", "ProfiledFunction"]
    );
    assert_eq!(
        runtime_call_target_rows().len(),
        runtime_targets.len(),
        "RuntimeCallTarget inventory drifted"
    );

    let terminators =
        declared_enum_variants(include_str!("../../ir/block.rs"), "pub enum Terminator");
    assert_eq!(
        terminators,
        [
            "Br",
            "CondBr",
            "Switch",
            "Return",
            "Throw",
            "Fatal",
            "GeneratorSuspend",
            "Unreachable",
        ]
    );
    assert_eq!(
        terminator_representatives().len(),
        terminators.len(),
        "Terminator inventory drifted"
    );

    let declared =
        declared_shape_predicates(include_str!("../capability.rs"));
    let mut inventoried: Vec<String> = shape_predicates()
        .iter()
        .map(|predicate| predicate.name.to_string())
        .collect();
    inventoried.sort();
    inventoried.dedup();
    assert_eq!(inventoried, declared, "shape predicate inventory drifted");
}

/// Verifies every Rust test identifier in the report names a checked-in test function.
#[test]
fn rust_test_identifiers_resolve_to_checked_in_tests() {
    let report = build_report();
    let rust_sources = [
        include_str!("../mod.rs"),
        include_str!("../capability.rs"),
        include_str!("../closures.rs"),
    ]
    .join("\n");
    let mut identifiers = Vec::new();
    identifiers.extend(report.tests.positive.iter().copied());
    identifiers.extend(report.tests.negative.iter().copied());
    identifiers.extend(report.tests.differential.iter().copied());
    identifiers.extend(report.tests.ownership.iter().copied());
    for family in report.families.values() {
        for row in &family.rows {
            if let Some(evidence) = &row.supported {
                identifiers.extend(evidence.tests.iter().copied());
            }
        }
    }

    for identifier in identifiers {
        let test_name = identifier
            .rsplit("::")
            .next()
            .expect("test identifier has a final segment");
        assert!(
            rust_sources.contains(&format!("fn {test_name}(")),
            "inventory references missing Rust test {identifier:?}"
        );
    }

    assert!(include_str!("../../../scripts/test-wasm-hosts.sh").contains("#!/"));
    assert!(
        include_str!("../../../.github/workflows/ci.yml").contains("wasm-host-portability:")
    );
}

/// Verifies the report rejects stale literal historical counts by deriving
/// totals from the enumeration rather than copying prose figures.
#[test]
fn totals_are_derived_not_copied_from_prose() {
    let report = build_report();
    assert!(report.totals.stale_literal_counts_rejected);
    let op = &report.families["op"];
    assert_eq!(op.total, Op::all().len());
    assert_eq!(op.total, op.supported + op.excluded + op.missing);
    let rt = &report.families["runtime_fn"];
    assert_eq!(rt.total, RuntimeFnId::all().len());
    let un = &report.families["unary_string"];
    assert_eq!(un.total, UnaryStringRuntime::all().len());
    // The supported count is derived from the capability classifier, not
    // copied from the spec prose's historical "90 of 236" figure.
    let derived_supported = Op::all()
        .iter()
        .copied()
        .filter(|o| op_is_supported(*o) && op_exclusion(*o).is_none())
        .count();
    assert_eq!(op.supported, derived_supported);
    let derived_excluded = Op::all()
        .iter()
        .copied()
        .filter(|o| op_exclusion(*o).is_some())
        .count();
    assert_eq!(op.excluded, derived_excluded);
    let _ = (rt.supported, un.missing);
}

/// Verifies the gate fails while any missing identity remains reachable,
/// matching the W0 rule that missing/reachable entries fail the gate.
#[test]
fn gate_fails_when_missing_reachable() {
    let report = build_report();
    assert!(report.totals.missing > 0, "current revision must still have missing identities");
    assert_eq!(report.totals.gate, "fail");
    assert!(
        report.totals.gate_reason.contains("missing"),
        "gate_reason should explain the missing count: {}",
        report.totals.gate_reason
    );
}

/// Verifies a dispatched lowerer is not reported as supported when every PHP shape is rejected.
#[test]
fn float_to_int_remains_missing_until_a_php_shape_is_admitted() {
    assert!(!op_is_supported(Op::FToI));
    let row = op_row(Op::FToI);
    assert_eq!(row.disposition, Disposition::Missing);
    assert!(row.supported.is_none());
}

/// Verifies excluded rows carry a complete contract and a matching target
/// diagnostic, so exclusions are never silently "unsupported".
#[test]
fn excluded_rows_carry_complete_contracts() {
    let report = build_report();
    let mut excluded = 0usize;
    for family in report.families.values() {
        for row in &family.rows {
            if row.disposition == Disposition::Excluded {
                excluded += 1;
                let exclusion = row.excluded.as_ref().expect("excluded row has contract");
                assert!(!exclusion.category.is_empty());
                assert!(!exclusion.reason.is_empty());
                assert!(!exclusion.owner.is_empty());
                assert!(!exclusion.removal_gate.is_empty());
                assert!(
                    !exclusion.diagnostic.is_empty(),
                    "excluded row {:?} lacks a matching diagnostic",
                    row.name
                );
            }
        }
    }
    assert!(excluded > 0, "expected native-only exclusions to be recorded");
}

/// Verifies native implementation requirements never exclude ordinary PHP builtins.
#[test]
fn ordinary_php_bridge_and_system_builtins_remain_reachable_gaps() {
    for id in [
        RuntimeFnId::Md5,
        RuntimeFnId::Hash,
        RuntimeFnId::Sha1,
        RuntimeFnId::MbStrlen,
        RuntimeFnId::Gzcompress,
    ] {
        assert!(
            runtime_fn_exclusion(id).is_none(),
            "ordinary PHP runtime {} must not be excluded",
            id.as_eir()
        );
        assert_eq!(runtime_fn_row(id).disposition, Disposition::Missing);
    }
    for id in [
        RuntimeFnId::Ptr,
        RuntimeFnId::BufferLen,
        RuntimeFnId::ZvalPack,
        RuntimeFnId::ClassAttributeNames,
        RuntimeFnId::Header,
    ] {
        assert_eq!(
            runtime_fn_row(id).disposition,
            Disposition::Excluded,
            "{} is an explicit Elephc/web exclusion",
            id.as_eir()
        );
    }
}

/// Verifies the report serializes to a well-formed JSON object whose
/// derived totals and per-family counts match the in-memory report, so the
/// committed JSON artifact is a faithful encoding.
#[test]
fn report_serializes_to_faithful_json() {
    let report = build_report();
    let json = serde_json::to_string(&report).expect("serialize report");
    let value: serde_json::Value =
        serde_json::from_str(&json).expect("report JSON parses");
    let obj = value.as_object().expect("report is a JSON object");
    assert_eq!(obj["metadata"]["schema"], SCHEMA_ID);
    assert_eq!(
        obj["metadata"]["pins"]["wasm_compliance_sha256"],
        FROZEN_SPEC_SHA256
    );
    assert_eq!(
        obj["metadata"]["pins"]["wasm_core_3_0"]["commit"],
        "9d36019973201a19f9c9ebb0f10828b2fe2374aa"
    );
    assert_eq!(obj["metadata"]["pins"]["php_src"].as_array().unwrap().len(), 4);
    assert_eq!(obj["metadata"]["pins"]["toolchain"]["wasmparser"], "0.252.0");
    let totals = &obj["totals"];
    assert_eq!(totals["total"], report.totals.total);
    assert_eq!(totals["supported"], report.totals.supported);
    assert_eq!(totals["excluded"], report.totals.excluded);
    assert_eq!(totals["missing"], report.totals.missing);
    assert_eq!(totals["evidence_gaps"], report.totals.evidence_gaps);
    assert_eq!(totals["gate"], report.totals.gate);
    assert_eq!(totals["stale_literal_counts_rejected"], true);
    for (name, family) in &report.families {
        let f = &obj["families"][name];
        assert_eq!(f["total"], family.total);
        assert_eq!(f["supported"], family.supported);
        assert_eq!(f["excluded"], family.excluded);
        assert_eq!(f["missing"], family.missing);
        assert_eq!(
            f["rows"].as_array().unwrap().len(),
            family.rows.len()
        );
    }
    assert!(validate_report(&report).is_empty());
}

/// Verifies revision metadata is either absent for the baseline or a full paired record.
#[test]
fn revision_metadata_rejects_partial_or_short_git_identity() {
    let mut report = build_report();
    report.metadata.commit = Some("abc".to_string());
    let errors = validate_report(&report);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("must either both be present")),
        "{errors:?}"
    );

    report.metadata.dirty = Some(false);
    let errors = validate_report(&report);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("full 40-hex Git commit")),
        "{errors:?}"
    );

    report.metadata.commit =
        Some("0123456789abcdef0123456789abcdef01234567".to_string());
    assert!(validate_report(&report).is_empty());
}

/// Verifies the human summary is non-empty and names the derived totals.
#[test]
fn human_summary_names_derived_totals() {
    let report = build_report();
    let summary = human_summary(&report);
    assert!(summary.contains("wasm32-wasi"));
    assert!(summary.contains("derived"));
    assert!(summary.contains(&report.totals.total.to_string()));
}
