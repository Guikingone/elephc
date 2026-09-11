//! Purpose:
//! Keeps the per-context exception state reachable through ONE door. Every access to
//! `_exc_value`, `_exc_handler_top` and `_exc_call_frame_top` must go through the `abi::`
//! symbol accessors, so that making the family per-context is a change to four functions
//! rather than to the 300 call sites that use them.
//!
//! Called from:
//! - `cargo test --bin elephc` only. `#[cfg(test)]`-gated at its `mod` declaration.
//!
//! Key details:
//! - WHY THIS EXISTS. A thread that throws must not walk the MAIN thread's handler chain,
//!   so `.plans/sandbox-threads.md` lists this family as the one that blocks M1. The
//!   migration is cheap exactly as long as the accessors are the only door: route them on
//!   the symbol name and no call site changes, with the linker as the tripwire (a ctx build
//!   does not declare the legacy symbols, so a missed path fails to link instead of reading
//!   another thread's state).
//! - WHAT A HAND-ROLLED ACCESS COSTS. It is invisible to that tripwire when it materializes
//!   the address separately: `emit_symbol_address(…, "x9", "_exc_value")` followed by a bare
//!   `str x0, [x9]` still names the symbol, but the store does not, so re-pointing the
//!   accessors would leave that store writing the process-global cell with nothing to say so.
//! - HOW THE INVENTORY WAS WRONG THREE TIMES. Grepping for the symbol beside an
//!   `emitter.instruction` found three sites and reported them as the whole residue. There
//!   were fifteen: twelve took the two-step form, which puts the symbol on the line BEFORE
//!   the access, and a thirteenth hand-rolled store hid behind a line break that separated
//!   `ctx.emitter` from `.instruction(`. Every one of those greps was written to match a
//!   SPELLING. This audit asks about the symbol instead, and found what they did not.
//! - THE CONVERSION CHANGED NO EMITTED BYTE. For a value in `x0` the accessor emits exactly
//!   what the hand-rolled pair did, in both the plain and the PIC path — measured by diffing
//!   `--emit-asm` output before and after.

use std::path::Path;

/// The state a thread must own before it can throw.
const PER_CONTEXT_EXCEPTION_SYMBOLS: &[&str] =
    &["_exc_value", "_exc_handler_top", "_exc_call_frame_top"];

/// The only functions allowed to name those symbols: the ones that perform the WHOLE access,
/// so that re-pointing them for M1 re-points everything.
///
/// `emit_symbol_address` is deliberately NOT here, though it is an `abi::` accessor too. It
/// hands back a raw address and the instruction that then uses it never names the symbol, so
/// a two-step access is invisible to any name-based check — including this one, and including
/// the linker tripwire M1 relies on. Twelve sites had that shape; all twelve are converted,
/// which is what lets the list be this strict.
const ALLOWED_ACCESSORS: &[&str] = &[
    "emit_load_symbol_to_reg",
    "emit_store_reg_to_symbol",
    "emit_store_zero_to_symbol",
    "emit_load_symbol_to_result",
];

/// Files that may name the symbols outside an accessor call, each with the reason.
///
/// `data/fixed.rs` DECLARES the storage — it has to write the names. `family_audit.rs` is
/// this file. Nothing else belongs here: an entry is a hole in the M1 migration.
const ALLOWED_FILES: &[&str] = &["data/fixed.rs", "exceptions/family_audit.rs"];

/// How many lines back to look for the start of a multi-line accessor call. Five covers
/// `emit_load_symbol_to_reg(` plus four arguments on their own lines, which is the widest
/// shape in the tree today.
const STATEMENT_LOOKBACK: usize = 5;

/// Walks `dir`, calling `visit` with every `.rs` file's path and contents.
fn for_each_rust_file(dir: &Path, visit: &mut impl FnMut(&Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            for_each_rust_file(&path, visit);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            if let Ok(text) = std::fs::read_to_string(&path) {
                visit(&path, &text);
            }
        }
    }
}

/// EVERY EXCEPTION-STATE ACCESS GOES THROUGH AN `abi::` ACCESSOR.
///
/// A line naming one of the symbols must sit inside a call to one of them. The unit is the
/// STATEMENT, not the line: accessor calls wrap, leaving the symbol alone on an argument line,
/// and judging line by line called five such calls violations on the first run.
///
/// The one thing this cannot see is an access through an address materialized elsewhere,
/// because the instruction that performs it never names the symbol. That is why
/// `emit_symbol_address` is not an allowed accessor: the shape is forbidden rather than
/// tolerated, which is the only way a name-based check can cover it.
#[test]
fn exception_state_is_only_reached_through_the_abi_accessors() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders: Vec<String> = Vec::new();
    for_each_rust_file(&src, &mut |path, text| {
        let display = path.to_string_lossy().replace('\\', "/");
        if ALLOWED_FILES.iter().any(|allowed| display.ends_with(allowed)) {
            return;
        }
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // A doc or ordinary comment naming the symbol is prose, not an access.
            if trimmed.starts_with("//") {
                continue;
            }
            // Only the code before a trailing comment can perform an access.
            let code = line.split("//").next().unwrap_or(line);
            if !PER_CONTEXT_EXCEPTION_SYMBOLS.iter().any(|symbol| code.contains(symbol)) {
                continue;
            }
            // An assertion READS emitted text; it cannot emit an access. Several tests check
            // that a lowering produced the right store, and naming the symbol is the point.
            if code.contains("assert") {
                continue;
            }
            // An accessor call often spans several lines, with the symbol on its own line as
            // an argument. Judging one line at a time called five such calls violations on the
            // first run. The statement is what matters, so look back to where it began — a
            // short window, because an accessor call is a handful of arguments, not a block.
            let window_start = index.saturating_sub(STATEMENT_LOOKBACK);
            let statement = lines[window_start..=index].join(" ");
            if ALLOWED_ACCESSORS.iter().any(|accessor| statement.contains(accessor)) {
                continue;
            }
            offenders.push(format!("{display}:{}: {}", index + 1, line.trim()));
        }
    });

    assert!(
        offenders.is_empty(),
        "these lines reach the per-context exception state without an abi:: accessor. Routing \
         the family for M1 re-points the accessors, so an access spelled by hand keeps writing \
         the process-global cell and no linker error says so:\n  {}",
        offenders.join("\n  ")
    );
}
