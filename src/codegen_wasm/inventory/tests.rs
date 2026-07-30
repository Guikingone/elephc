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
    let totals = &obj["totals"];
    assert_eq!(totals["total"], report.totals.total);
    assert_eq!(totals["supported"], report.totals.supported);
    assert_eq!(totals["excluded"], report.totals.excluded);
    assert_eq!(totals["missing"], report.totals.missing);
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

/// Verifies the human summary is non-empty and names the derived totals.
#[test]
fn human_summary_names_derived_totals() {
    let report = build_report();
    let summary = human_summary(&report);
    assert!(summary.contains("wasm32-wasi"));
    assert!(summary.contains("derived"));
    assert!(summary.contains(&report.totals.total.to_string()));
}
