//! Purpose:
//! Standalone W0 inventory exporter that prints the deterministic
//! `codegen_wasm::inventory` report as JSON, plus a `--summary` human summary.
//!
//! Called from:
//! - `cargo run --example gen_wasm_inventory` (W0 evidence generation / CI gate).
//!
//! Key details:
//! - Mirrors `tools/gen_builtins.rs`: declared as an example so it can read the
//!   `elephc` library's `codegen_wasm::inventory` API without linking it into
//!   the `elephc` binary.
//! - The committed baseline leaves `metadata.commit`/`dirty` as `null`; pass
//!   `--with-revision` to fill them from `git` for a per-run CI manifest.

use std::process::Command;

/// Prints the WASM capability inventory JSON (or `--summary` text) to stdout.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let want_summary = args.iter().any(|a| a == "--summary");
    let with_revision = args.iter().any(|a| a == "--with-revision");

    let mut report = elephc::codegen_wasm::inventory::build_report();
    if with_revision {
        let commit = git_output(&["rev-parse", "HEAD"]);
        let dirty = !git_output(&["status", "--porcelain"]).is_empty();
        report.metadata.commit = Some(commit);
        report.metadata.dirty = Some(dirty);
    }

    if want_summary {
        println!("{}", elephc::codegen_wasm::inventory::human_summary(&report));
        return;
    }

    let errors = elephc::codegen_wasm::inventory::validate_report(&report);
    if !errors.is_empty() {
        eprintln!("WASM inventory schema validation failed:");
        for error in &errors {
            eprintln!("  - {error}");
        }
        std::process::exit(1);
    }

    let json = serde_json::to_string_pretty(&report).expect("serialize inventory report");
    println!("{json}");
}

/// Runs a `git` command and returns its trimmed stdout (empty on failure).
fn git_output(args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}
