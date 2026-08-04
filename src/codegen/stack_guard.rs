//! Purpose:
//! Emits the per-function half of the call-stack overflow guard: the prologue compare of
//! the stack pointer against the runtime `_stack_limit` floor, and the process-entry call
//! that publishes that floor.
//!
//! Called from:
//! - `crate::codegen::frame::emit_function_prologue_with_label()` for every compiled PHP
//!   function, method, closure, and generator body.
//! - `crate::codegen::frame::emit_main_prologue()` and
//!   `crate::codegen::frame::emit_web_entry_stub()` for the one-time floor measurement.
//!
//! Key details:
//! - The check runs immediately after the frame has been reserved, so the compare already
//!   accounts for this function's own frame; the runtime reserve only has to cover what a
//!   single guarded frame can still consume before the next guarded call.
//! - It must be branch-only and must not touch memory below the stack pointer, because it
//!   runs when the remaining stack may be a single page.
//! - AArch64 keeps the conditional branch local and reaches `__rt_stack_overflow` with an
//!   unconditional `b`: `b.cond` only encodes a ±1 MiB displacement, which large programs
//!   exceed, while `b` reaches ±128 MiB and gets linker veneers beyond that.
//! - Registers: only x9 (AArch64 symbol scratch) is clobbered, and nothing at all on
//!   x86_64 outside PIC mode. Incoming argument registers are untouched, which is what
//!   lets the check sit before the parameter spill loop.

use crate::codegen::abi;
use crate::codegen::emit::Emitter;
use crate::codegen::platform::Arch;
use crate::codegen_support::runtime::STACK_LIMIT_SYMBOL;

/// Runtime symbol that measures the stack once and publishes `_stack_limit`.
const STACK_LIMIT_INIT_SYMBOL: &str = "__rt_stack_limit_init";

/// Runtime symbol that reports the controlled fatal and exits with status 255.
const STACK_OVERFLOW_SYMBOL: &str = "__rt_stack_overflow";

/// Emits the one-time call that measures the running stack and publishes the guard floor.
///
/// Must be emitted after the process-entry prologue has already stored argc/argv, because
/// the helper is an ordinary call and clobbers the C-ABI argument registers. Until it runs,
/// `_stack_limit` is zero and every prologue check passes.
pub(super) fn emit_stack_limit_init_call(emitter: &mut Emitter) {
    emitter.comment("publish the call-stack overflow floor for this process");
    abi::emit_call_label(emitter, STACK_LIMIT_INIT_SYMBOL);
}

/// Emits the prologue stack-depth check for one compiled function.
///
/// Compares the stack pointer against `_stack_limit` as an unsigned value and branches to
/// `__rt_stack_overflow` when it is below. A zero limit (the pre-initialization state, and
/// the state whenever the floor could not be determined) makes the compare always pass, so
/// the guard is inert rather than wrong when the bounds are unknown.
///
/// `ok_label` must be a function-unique label; it is emitted immediately after the check on
/// AArch64 and unused on x86_64, whose `jb rel32` reaches the runtime symbol directly.
pub(super) fn emit_stack_limit_check(emitter: &mut Emitter, ok_label: &str) {
    emitter.comment("call-stack overflow guard");
    match emitter.target.arch {
        Arch::AArch64 => {
            if emitter.pic_data_refs {
                // PIC builds must reach the limit through the GOT; the shared helper owns
                // that sequence and leaves the comparison flags set the same way.
                abi::emit_cmp_reg_to_symbol(emitter, "sp", STACK_LIMIT_SYMBOL);
            } else {
                emitter.adrp("x9", STACK_LIMIT_SYMBOL);
                emitter.ldr_lo12("x9", "x9", STACK_LIMIT_SYMBOL);
                emitter.instruction("cmp sp, x9");                              // is the freshly reserved frame below the published stack floor?
            }
            emitter.instruction(&format!("b.hs {}", ok_label));                 // still above the floor — continue into the function body
            emitter.instruction(&format!("b {}", STACK_OVERFLOW_SYMBOL));       // out of stack — report the controlled fatal and exit
            emitter.label(ok_label);
        }
        Arch::X86_64 => {
            abi::emit_cmp_reg_to_symbol(emitter, "rsp", STACK_LIMIT_SYMBOL);
            emitter.instruction(&format!("jb {}", STACK_OVERFLOW_SYMBOL));      // out of stack — report the controlled fatal and exit
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::platform::{Platform, Target};

    /// Emits the prologue check for `target` and returns the generated assembly text.
    fn check_asm(target: Target) -> String {
        let mut emitter = Emitter::new(target);
        emit_stack_limit_check(&mut emitter, "_test_stack_ok");
        emitter.output()
    }

    /// macos-aarch64 must compare the stack pointer against `_stack_limit` and reach the
    /// fatal through an unconditional branch, so the conditional branch stays local and
    /// cannot exceed the AArch64 ±1 MiB `b.cond` displacement in large programs.
    #[test]
    fn test_prologue_check_macos_aarch64() {
        let asm = check_asm(Target::new(Platform::MacOS, Arch::AArch64));
        assert!(asm.contains("adrp x9, _stack_limit@PAGE"), "{asm}");
        assert!(asm.contains("ldr x9, [x9, _stack_limit@PAGEOFF]"), "{asm}");
        assert!(asm.contains("cmp sp, x9"), "{asm}");
        assert!(asm.contains("b.hs _test_stack_ok"), "{asm}");
        assert!(asm.contains("b __rt_stack_overflow"), "{asm}");
        assert!(asm.contains("_test_stack_ok:"), "{asm}");
    }

    /// linux-aarch64 emits the same guard through the ELF `:lo12:` relocation spelling.
    #[test]
    fn test_prologue_check_linux_aarch64() {
        let asm = check_asm(Target::new(Platform::Linux, Arch::AArch64));
        assert!(asm.contains("adrp x9, _stack_limit"), "{asm}");
        assert!(asm.contains("ldr x9, [x9, :lo12:_stack_limit]"), "{asm}");
        assert!(asm.contains("cmp sp, x9"), "{asm}");
        assert!(asm.contains("b.hs _test_stack_ok"), "{asm}");
        assert!(asm.contains("b __rt_stack_overflow"), "{asm}");
    }

    /// linux-x86_64 folds the whole guard into two instructions: a RIP-relative memory
    /// compare and a `jb`, whose rel32 displacement always reaches the runtime symbol.
    #[test]
    fn test_prologue_check_linux_x86_64() {
        let asm = check_asm(Target::new(Platform::Linux, Arch::X86_64));
        assert!(
            asm.contains("cmp rsp, QWORD PTR [rip + _stack_limit]"),
            "{asm}"
        );
        assert!(asm.contains("jb __rt_stack_overflow"), "{asm}");
    }

    /// The check must not write memory or touch an argument register: it runs on a frame
    /// that may be one page away from the guard page, and before the incoming parameters
    /// have been spilled to their slots.
    #[test]
    fn test_prologue_check_touches_no_memory_or_argument_registers() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new(Platform::Linux, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
        ] {
            let asm = check_asm(target);
            for line in asm.lines().map(str::trim) {
                assert!(
                    !line.starts_with("str ") && !line.starts_with("stp ")
                        && !line.starts_with("stur ") && !line.starts_with("push "),
                    "guard stored to memory on {target:?}: {line}"
                );
            }
            for arg_reg in ["x0", "x1", "x2", "rdi", "rsi", "rdx", "rcx", "r8", "r9"] {
                assert!(
                    !asm.contains(&format!(" {arg_reg},")),
                    "guard clobbered {arg_reg} on {target:?}: {asm}"
                );
            }
        }
    }

    /// The one-time initializer must be a plain call to the runtime measurement helper.
    #[test]
    fn test_stack_limit_init_call_targets_the_runtime_helper() {
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_stack_limit_init_call(&mut emitter);
        assert!(emitter.output().contains("call __rt_stack_limit_init"));
    }
}
