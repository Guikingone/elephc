//! Purpose:
//! Relaxes only AArch64 conditional branches whose generated targets exceed the
//! architecture's short branch encodings.
//!
//! Called from:
//! - `crate::codegen::finalize_user_asm()` after user text and metadata are complete.
//!
//! Key details:
//! - Symbolic source distance is a conservative proxy for instruction distance;
//!   nearby branches remain compact, while far branches use an inverse-condition island.

use std::collections::HashMap;
use std::fmt::Write;

use crate::codegen::platform::{Arch, Platform, Target};

const CONDITIONAL_SOURCE_DISTANCE: usize = 768 * 1024;
const TEST_BIT_SOURCE_DISTANCE: usize = 24 * 1024;
const WIDE_BRANCH_SOURCE_DISTANCE: usize = 96 * 1024 * 1024;

/// Parsed symbolic AArch64 conditional branch.
struct ConditionalBranch<'a> {
    inverse_mnemonic: &'static str,
    leading_operands: &'a str,
    target: &'a str,
    source_limit: usize,
}

/// Rewrites only symbolic AArch64 conditional branches whose targets are conservatively far.
pub(super) fn relax_conditional_branches(assembly: String, target: Target) -> String {
    if target.arch != Arch::AArch64 {
        return assembly;
    }

    let mut labels = HashMap::new();
    let mut source_offset = 0usize;
    for line in assembly.split_inclusive('\n') {
        let trimmed = line.trim();
        if let Some(label) = trimmed.strip_suffix(':') {
            if is_symbol(label) {
                labels.insert(label, source_offset);
            }
        }
        source_offset += line.len();
    }

    let mut relaxed = None::<String>;
    source_offset = 0;
    for line in assembly.split_inclusive('\n') {
        let expansion = parse_conditional_branch(line).and_then(|branch| {
            let target_offset = labels.get(branch.target).copied()?;
            let distance = source_offset.abs_diff(target_offset);
            (distance > branch.source_limit).then_some((branch, distance))
        });

        if let Some((branch, distance)) = expansion {
            let output = relaxed.get_or_insert_with(|| {
                let mut output = String::with_capacity(assembly.len() + 4096);
                output.push_str(&assembly[..source_offset]);
                output
            });
            emit_relaxed_branch(output, &branch, target, source_offset, distance);
        } else if let Some(output) = relaxed.as_mut() {
            output.push_str(line);
        }
        source_offset += line.len();
    }

    drop(labels);
    relaxed.unwrap_or(assembly)
}

/// Parses the condition, preserved operands, and symbolic target from one assembly line.
fn parse_conditional_branch(line: &str) -> Option<ConditionalBranch<'_>> {
    let trimmed = line.trim();
    let (mnemonic, operands) = trimmed.split_once(char::is_whitespace)?;
    let operands = operands.trim();

    let branch = if let Some(condition) = mnemonic.strip_prefix("b.") {
        let inverse_mnemonic = match condition {
            "eq" => "b.ne",
            "ne" => "b.eq",
            "cs" => "b.cc",
            "cc" => "b.cs",
            "hs" => "b.lo",
            "lo" => "b.hs",
            "mi" => "b.pl",
            "pl" => "b.mi",
            "vs" => "b.vc",
            "vc" => "b.vs",
            "hi" => "b.ls",
            "ls" => "b.hi",
            "ge" => "b.lt",
            "lt" => "b.ge",
            "gt" => "b.le",
            "le" => "b.gt",
            _ => return None,
        };
        ConditionalBranch {
            inverse_mnemonic,
            leading_operands: "",
            target: operands,
            source_limit: CONDITIONAL_SOURCE_DISTANCE,
        }
    } else if matches!(mnemonic, "cbz" | "cbnz") {
        let (register, target) = operands.split_once(',')?;
        ConditionalBranch {
            inverse_mnemonic: if mnemonic == "cbz" { "cbnz" } else { "cbz" },
            leading_operands: register.trim(),
            target: target.trim(),
            source_limit: CONDITIONAL_SOURCE_DISTANCE,
        }
    } else if matches!(mnemonic, "tbz" | "tbnz") {
        let (register_and_bit, target) = operands.rsplit_once(',')?;
        ConditionalBranch {
            inverse_mnemonic: if mnemonic == "tbz" { "tbnz" } else { "tbz" },
            leading_operands: register_and_bit.trim(),
            target: target.trim(),
            source_limit: TEST_BIT_SOURCE_DISTANCE,
        }
    } else {
        return None;
    };

    is_symbol(branch.target).then_some(branch)
}

/// Emits an inverse-condition island and selects a register-free or absolute far jump.
fn emit_relaxed_branch(
    output: &mut String,
    branch: &ConditionalBranch<'_>,
    target: Target,
    branch_offset: usize,
    distance: usize,
) {
    let skip_label = match target.platform {
        Platform::MacOS => format!("L_elephc_relaxed_branch_{}", branch_offset),
        Platform::Linux | Platform::Windows => {
            format!(".L_elephc_relaxed_branch_{}", branch_offset)
        }
    };
    if branch.leading_operands.is_empty() {
        let _ = writeln!(output, "    {} {}", branch.inverse_mnemonic, skip_label);
    } else {
        let _ = writeln!(
            output,
            "    {} {}, {}",
            branch.inverse_mnemonic, branch.leading_operands, skip_label
        );
    }

    if distance <= WIDE_BRANCH_SOURCE_DISTANCE {
        let _ = writeln!(output, "    b {}", branch.target);
    } else {
        match target.platform {
            Platform::MacOS => {
                let _ = writeln!(output, "    adrp x16, {}@PAGE", branch.target);
                let _ = writeln!(output, "    add x16, x16, {}@PAGEOFF", branch.target);
            }
            Platform::Linux => {
                let _ = writeln!(output, "    adrp x16, {}", branch.target);
                let _ = writeln!(output, "    add x16, x16, :lo12:{}", branch.target);
            }
            Platform::Windows => unreachable!("AArch64 Windows codegen is unsupported"),
        }
        let _ = writeln!(output, "    br x16");
    }
    let _ = writeln!(output, "{}:", skip_label);
}

/// Returns whether a branch operand is a plain symbolic assembly label.
fn is_symbol(value: &str) -> bool {
    value
        .bytes()
        .next()
        .is_some_and(|first| !first.is_ascii_digit())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'$'))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies nearby conditional branches remain byte-for-byte compact.
    #[test]
    fn nearby_conditional_branch_is_unchanged() {
        let assembly = "    b.eq target\n    nop\ntarget:\n".to_string();
        let target = Target::new(Platform::MacOS, Arch::AArch64);
        assert_eq!(relax_conditional_branches(assembly.clone(), target), assembly);
    }

    /// Verifies a distant condition-code branch uses a register-free branch island.
    #[test]
    fn distant_condition_branch_is_relaxed() {
        let mut assembly = "    b.eq target\n".to_string();
        assembly.push_str(&"    nop\n".repeat(CONDITIONAL_SOURCE_DISTANCE / 8 + 2));
        assembly.push_str("target:\n");
        let target = Target::new(Platform::MacOS, Arch::AArch64);
        let relaxed = relax_conditional_branches(assembly, target);
        assert!(relaxed.starts_with(
            "    b.ne L_elephc_relaxed_branch_0\n    b target\nL_elephc_relaxed_branch_0:\n"
        ));
    }

    /// Verifies bit-test operands and Linux-local label syntax survive relaxation.
    #[test]
    fn distant_bit_test_branch_preserves_operands() {
        let mut assembly = "    tbnz x4, #63, target\n".to_string();
        assembly.push_str(&"    nop\n".repeat(TEST_BIT_SOURCE_DISTANCE / 8 + 2));
        assembly.push_str("target:\n");
        let target = Target::new(Platform::Linux, Arch::AArch64);
        let relaxed = relax_conditional_branches(assembly, target);
        assert!(relaxed.starts_with(
            "    tbz x4, #63, .L_elephc_relaxed_branch_0\n    b target\n.L_elephc_relaxed_branch_0:\n"
        ));
    }

    /// Verifies non-AArch64 assembly is returned unchanged.
    #[test]
    fn other_architecture_is_unchanged() {
        let assembly = "    b.eq target\ntarget:\n".to_string();
        let target = Target::new(Platform::Linux, Arch::X86_64);
        assert_eq!(relax_conditional_branches(assembly.clone(), target), assembly);
    }
}
