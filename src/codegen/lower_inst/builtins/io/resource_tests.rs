//! Purpose:
//! Assembly regression tests for closed-resource identity sentinels.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::io`.
//!
//! Key details:
//! - Preserves target-aware ABI handling, runtime calls, and result ownership.

use super::*;

#[cfg(test)]
mod resource_release_sentinel_tests {
    use super::emit_resource_release_sentinel;
    use crate::codegen::emit::Emitter;
    use crate::codegen::platform::{Arch, Platform, Target};

    /// Emits the release sentinel for one target and returns the assembly text.
    fn emit_for(target: Target) -> String {
        let mut emitter = Emitter::new(target);
        emit_resource_release_sentinel(&mut emitter);
        emitter.output()
    }

    /// Pins the AArch64 stamp: look the id up BEFORE overwriting the payload, store the
    /// negated id, and hand the native handle back to the caller's close dispatch.
    ///
    /// The bare `mov x10, #-1` this replaced erased the registry key, so every later
    /// display of the closed handle missed the table and minted a fresh id — PHP 8.5.6
    /// keeps `Resource id #5` and `get_resource_id($r) === 5` after `fclose($r)`.
    #[test]
    fn aarch64_stamps_the_negated_resource_id() {
        let asm = emit_for(Target::new(Platform::MacOS, Arch::AArch64));
        assert!(asm.contains("ldr x9, [sp], #16"), "{asm}");
        assert!(asm.contains("mov x11, x0"), "{asm}");
        assert!(asm.contains("bl __rt_resource_id_of"), "{asm}");
        assert!(asm.contains("neg x10, x0"), "{asm}");
        assert!(asm.contains("str x10, [x9, #8]"), "{asm}");
        assert!(asm.contains("mov x0, x11"), "{asm}");
        assert!(
            !asm.contains("mov x10, #-1"),
            "the identity-erasing bare -1 sentinel must not come back:\n{asm}"
        );
    }

    /// Pins the same stamp on x86_64. The stash slot is released BEFORE the call so the
    /// helper runs on the frame's own alignment, and `r10`/`r11` carry the handle and the
    /// box pointer across it — both are pushed and popped by `__rt_resource_id_of`.
    #[test]
    fn x86_64_stamps_the_negated_resource_id() {
        let asm = emit_for(Target::new(Platform::Linux, Arch::X86_64));
        assert!(asm.contains("mov r11, QWORD PTR [rsp]"), "{asm}");
        assert!(asm.contains("add rsp, 16"), "{asm}");
        assert!(asm.contains("mov r10, rax"), "{asm}");
        assert!(asm.contains("call __rt_resource_id_of"), "{asm}");
        assert!(asm.contains("neg rax"), "{asm}");
        assert!(asm.contains("mov QWORD PTR [r11 + 8], rax"), "{asm}");
        assert!(asm.contains("mov rax, r10"), "{asm}");
        assert!(
            !asm.contains("mov QWORD PTR [r11 + 8], -1"),
            "the identity-erasing bare -1 sentinel must not come back:\n{asm}"
        );
    }

    /// The stamped payload must stay NEGATIVE on both targets, because that is the only
    /// property the three `__rt_mixed_free_deep` resource arms rely on: they skip any
    /// payload at or above the UNSIGNED threshold `0x40000000`, and every negative value
    /// is unsigned-huge. A sentinel that stopped being negative would let scope cleanup
    /// close an already-closed descriptor a second time.
    #[test]
    fn the_sentinel_stays_negative_on_both_targets() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
        ] {
            let asm = emit_for(target);
            let negates = asm.contains("neg x10, x0") || asm.contains("neg rax");
            assert!(negates, "the stamped payload must be a negated id ({target:?}):\n{asm}");
        }
    }
}
