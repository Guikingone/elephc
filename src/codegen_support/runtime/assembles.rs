//! Purpose:
//! Feeds the fully generated runtime to a real assembler, for every target the compiler
//! emits, and fails on the first diagnostic. It answers the one question no text-level
//! audit can: does what we emit actually assemble?
//!
//! Called from:
//! - `cargo test --bin elephc` only. `#[cfg(test)]`-gated at its `mod` declaration.
//!
//! Key details:
//! - WHY THIS EXISTS. The `r14` → scratch migration renamed `movzx r14d, BYTE PTR [...]`
//!   into `movzx rsid, BYTE PTR [...]`. `r14d` is r14's 32-bit view, but rsi's is `esi` —
//!   `rsid` is not a register name at all. Every emitted-text test stayed green (the text
//!   was exactly what they asserted), the host suite stayed green (macOS assembles the
//!   AArch64 runtime, never the x86_64 one), and a container probe stayed green too,
//!   because the broken helper sits behind a feature its small program never enabled.
//!   The first thing to notice was a linux-x86_64 CI shard: `operand size mismatch for
//!   'movzx'`, 41 failed tests, in a job about PostgreSQL.
//! - A TEXT ASSERTION CANNOT CATCH THIS. Asserting the emitted string is asserting our own
//!   spelling back to ourselves. Only an assembler knows that `rsid` is not a register,
//!   that an immediate cannot be a store's source on AArch64, or that a displacement is out
//!   of range. This module borrows one.
//! - ALL FEATURES, EVERY TARGET. A feature-gated family is exactly where an invalid
//!   instruction hides from the shards that do not enable it, which is what happened here.
//! - CROSS-ASSEMBLY. clang's integrated assembler handles every target this compiler emits,
//!   from any host, so this runs on a developer machine rather than only in CI. The
//!   compiler already requires clang to link.

use std::io::Write;
use std::process::Command;

use crate::codegen_support::driver_support::generate_runtime_with_features;
use crate::codegen_support::platform::{Arch, Platform, Target};
use crate::codegen_support::runtime_features::RuntimeFeatures;

/// The clang target triple used to assemble each emitted target.
fn clang_triple(target: Target) -> &'static str {
    match (target.platform, target.arch) {
        (Platform::MacOS, Arch::AArch64) => "arm64-apple-macos",
        (Platform::MacOS, Arch::X86_64) => "x86_64-apple-macos",
        (Platform::Linux, Arch::AArch64) => "aarch64-unknown-linux-gnu",
        (Platform::Linux, Arch::X86_64) => "x86_64-unknown-linux-gnu",
        (Platform::Windows, _) => "x86_64-pc-windows-gnu",
    }
}

/// Assembles `asm` for `target`, returning the assembler's diagnostics on failure.
fn assemble(target: Target, asm: &str) -> Result<(), String> {
    let dir = std::env::temp_dir().join(format!(
        "elephc_runtime_assembles_{}_{}",
        std::process::id(),
        clang_triple(target).replace('-', "_")
    ));
    std::fs::create_dir_all(&dir).expect("failed to create the assembly scratch directory");
    let source = dir.join("runtime.s");
    let mut file = std::fs::File::create(&source).expect("failed to write the runtime assembly");
    file.write_all(asm.as_bytes())
        .expect("failed to write the runtime assembly");
    drop(file);

    let output = Command::new("clang")
        .arg("-c")
        .args(["-target", clang_triple(target)])
        // Intel syntax is what the x86_64 emitters write; the flag is ignored elsewhere.
        .arg("-masm=intel")
        .arg("-o")
        .arg(dir.join("runtime.o"))
        .arg(&source)
        .output()
        .expect("clang is required to assemble the runtime (it already links every build)");

    let result = if output.status.success() {
        Ok(())
    } else {
        // Keep the source when it failed: the diagnostics carry line numbers into it.
        return Err(format!(
            "assembling the {:?}/{:?} runtime failed (source kept at {}):\n{}",
            target.platform,
            target.arch,
            source.display(),
            String::from_utf8_lossy(&output.stderr),
        ));
    };
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// The generated runtime must assemble for every target, with every feature on.
///
/// One test per target would report only the first failure; this reports all of them,
/// because a mistake in a shared emitter usually breaks more than one.
#[test]
fn the_generated_runtime_assembles_for_every_target() {
    let targets = [
        Target::new(Platform::MacOS, Arch::AArch64),
        Target::new(Platform::Linux, Arch::AArch64),
        Target::new(Platform::Linux, Arch::X86_64),
    ];
    let mut failures: Vec<String> = Vec::new();
    for target in targets {
        for ctx_register in [false, true] {
            let features = RuntimeFeatures {
                ctx_register,
                ..RuntimeFeatures::all()
            };
            let asm = generate_runtime_with_features(8 * 1024 * 1024, target, features);
            if let Err(diagnostics) = assemble(target, &asm) {
                failures.push(format!("[ctx_register={ctx_register}] {diagnostics}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "the generated runtime must assemble:\n{}",
        failures.join("\n\n")
    );
}

/// Negative control: the audit must reject an instruction a text assertion would accept.
///
/// `rsid` is the exact shape that shipped — a register suffix that is valid for r8-r15
/// pasted onto a register whose 32-bit view has a different name entirely.
#[test]
fn the_assembler_gate_rejects_an_invalid_register_name() {
    let target = Target::new(Platform::Linux, Arch::X86_64);
    let asm = ".text\n.globl __rt_probe\n__rt_probe:\n    movzx rsid, BYTE PTR [rax + r9]\n    ret\n";
    assert!(
        assemble(target, asm).is_err(),
        "the gate must reject `movzx rsid, …`; if clang accepted it, the gate proves nothing"
    );
}
