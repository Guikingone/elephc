//! Purpose:
//! Adapts boxed eval calls to the same native heap metrics as compiled PHP.
//!
//! Called from:
//! - Both architecture arms of the versioned runtime builtin dispatcher.
//!
//! Key details:
//! - Samples counters before allocating the boxed return value.
//! - Optional flags accept scalar boolean coercion and reject containers.

use super::*;
use elephc_builtin_contract::RuntimeBuiltinId;

/// Stable runtime IDs and helper suffixes for the native heap metric family.
pub(super) const OPERATIONS: &[(RuntimeBuiltinId, &str)] = &[
    (RuntimeBuiltinId::MemoryGetUsage, "memory_get_usage"),
    (RuntimeBuiltinId::MemoryGetPeakUsage, "memory_get_peak_usage"),
    (RuntimeBuiltinId::MemoryResetPeakUsage, "memory_reset_peak_usage"),
    (RuntimeBuiltinId::MemoryGetAllocations, "memory_get_allocations"),
    (RuntimeBuiltinId::MemoryGetFrees, "memory_get_frees"),
];

/// Emits arity checks, flag conversion, metric sampling, and ownership transfer.
pub(super) fn emit_cases(emitter: &mut Emitter) {
    let arm = emitter.target.arch == Arch::AArch64;
    let suffix = if arm { "" } else { "_x86" };
    let unsupported = format!("__elephc_runtime_builtin_v1_unsupported{suffix}");
    let fatal = format!("__elephc_runtime_builtin_v1_fatal{suffix}");
    for &(id, name) in OPERATIONS {
        let label = format!("__elephc_runtime_builtin_v1_{name}{suffix}");
        emitter.label(&label);
        let flag = matches!(id, RuntimeBuiltinId::MemoryGetUsage | RuntimeBuiltinId::MemoryGetPeakUsage);
        let reset = id == RuntimeBuiltinId::MemoryResetPeakUsage;
        // -- validate PHP arity before reading any borrowed argument --
        if arm {
            emitter.instruction(&format!("cmp x21, #{}", usize::from(flag)));   // check the maximum supported argument count
            emitter.instruction(&format!("b.hi {unsupported}"));                // reject extra arguments without changing heap state
        } else {
            emitter.instruction(&format!("cmp r13, {}", usize::from(flag)));    // check the maximum supported argument count
            emitter.instruction(&format!("ja {unsupported}"));                  // reject extra arguments without sampling counters
        }
        if flag {
            if arm {
                emitter.instruction("mov x0, #0");                              // default real_usage to false without allocating a cell
                emitter.instruction(&format!("cbz x21, {label}_sample"));       // omitted flags skip argument conversion
                emitter.instruction("ldr x0, [x20]");                           // borrow the provided real_usage cell
            } else {
                emitter.instruction("xor edi, edi");                            // default real_usage to false
                emitter.instruction("test r13, r13");                           // detect the optional flag's presence
                emitter.instruction(&format!("jz {label}_sample"));             // omitted flags skip cell inspection
                emitter.instruction("mov rdi, QWORD PTR [r12]");                // borrow the provided real_usage cell
            }
            emitter.bl_c("__elephc_eval_value_type_tag");
            // -- permit only scalar inputs accepted by a weak PHP boolean parameter --
            if arm {
                emitter.instruction("cmp x0, #3");                              // integers, floats, strings, and booleans have tags zero through three
                emitter.instruction(&format!("b.ls {label}_flag"));             // scalar flags can use ordinary PHP truthiness
                emitter.instruction("cmp x0, #8");                              // null also coerces to false in weak calls
                emitter.instruction(&format!("b.ne {fatal}"));                  // arrays, objects, and resources cannot be boolean parameters
                emitter.label(&format!("{label}_flag"));
                emitter.instruction("ldr x0, [x20]");                           // reload the borrowed flag after tag inspection
            } else {
                emitter.instruction("cmp rax, 3");                              // scalar tags zero through three admit weak boolean coercion
                emitter.instruction(&format!("jbe {label}_flag"));              // accept the scalar flag without a temporary result box
                emitter.instruction("cmp rax, 8");                              // null coerces to false for weak builtin calls
                emitter.instruction(&format!("jne {fatal}"));                   // reject non-scalar flags
                emitter.label(&format!("{label}_flag"));
                emitter.instruction("mov rdi, QWORD PTR [r12]");                // reload the original borrowed flag cell
            }
            emitter.bl_c("__elephc_eval_value_truthy");
            if !arm {
                emitter.instruction("mov rdi, rax");                            // pass the normalized boolean through the SysV argument register
            }
            emitter.label(&format!("{label}_sample"));
        }
        abi::emit_call_label(emitter, &format!("__rt_{name}"));
        if reset {
            emitter.bl_c("__elephc_eval_value_null");
        } else {
            if !arm {
                emitter.instruction("mov rdi, rax");                            // preserve the sampled integer before allocating its result cell
            }
            emitter.bl_c("__elephc_eval_value_int");
        }
        emitter.instruction(&format!("{} __elephc_runtime_builtin_v1_result{suffix}", if arm { "b" } else { "jmp" })); // transfer exactly one fresh result cell
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::Target;
    use std::io::Write;
    use std::process::{Command, Stdio};

    /// Assembles the native and boxed metric boundaries for every supported object format.
    #[test]
    #[ignore = "requires clang with ELF and Apple AArch64 assembler support"]
    fn memory_metrics_assemble_all_targets() {
        for (name, triple) in [
            ("macos-aarch64", "arm64-apple-macos11"),
            ("ios-arm64", "arm64-apple-ios13"),
            ("ios-sim-arm64", "arm64-apple-ios13-simulator"),
            ("linux-aarch64", "aarch64-linux-gnu"),
            ("linux-x86_64", "x86_64-linux-gnu"),
        ] {
            for pic in [false, true] {
                let target = Target::parse(name).unwrap();
                let mut emitter = Emitter::new(target);
                emitter.pic_data_refs = pic;
                if target.arch == Arch::X86_64 { emitter.raw(".intel_syntax noprefix"); }
                emitter.raw(".text");
                crate::codegen_support::runtime::arrays::emit_memory(&mut emitter);
                emit_cases(&mut emitter);
                // -- complete the dispatcher's shared exits for this assembly-only fixture --
                let suffix = if target.arch == Arch::AArch64 { "" } else { "_x86" };
                for exit in ["unsupported", "fatal", "result"] {
                    emitter.label(&format!("__elephc_runtime_builtin_v1_{exit}{suffix}"));
                    emitter.instruction("ret");                                 // terminate the standalone fixture without a missing branch target
                }
                let mut assembler = Command::new("clang")
                    .args(["-target", triple, "-x", "assembler", "-c", "-", "-o", "/dev/null"])
                    .stdin(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("clang is required");
                assembler.stdin.take().unwrap().write_all(emitter.output().as_bytes()).unwrap();
                let result = assembler.wait_with_output().unwrap();
                assert!(result.status.success(), "{name}, pic={pic}: {}", String::from_utf8_lossy(&result.stderr));
            }
        }
    }
}
