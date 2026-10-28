//! Purpose:
//! Bounds producer result tags and recursive packing before conversion reaches the PHP host.
//!
//! Called from:
//! - The two target implementations of `__rt_zval_pack_element`.
//!
//! Key details:
//! - Only the producer profile references this request-local counter.
//! - Conversion failures roll back the native image instead of recursively retiring partial results.
//! - Unsupported tags fail before pointer normalization can erase their identity.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Interposes a bounded wrapper before the ordinary value packer implementation.
pub(super) fn emit_guard(emitter: &mut Emitter) {
    const DEPTH: &str = "_elephc_phpext_pack_depth";
    if !emitter.phpext_output { return; }
    let full = "L_phpext_pack_invalid_result";
    let scratch = abi::secondary_scratch_reg(emitter);
    abi::emit_frame_prologue(emitter, 16);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("cmp x0, #8");                                  // admit known PHP value tags through null before pointer normalization
            emitter.instruction(&format!("b.hi {full}"));                       // reject callables, resources and raw pointer tags at the host boundary
        }
        Arch::X86_64 => {
            emitter.instruction("cmp rax, 8");                                  // admit known PHP value tags through null before pointer normalization
            emitter.instruction(&format!("ja {full}"));                         // reject callables, resources and raw pointer tags at the host boundary
        }
    }
    abi::emit_load_symbol_to_reg(emitter, scratch, DEPTH, 0);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("cmp {scratch}, #65"));                // allow 64 nested arrays and their terminal scalar value
            emitter.instruction(&format!("b.hs {full}"));                       // reject excessive recursion before packing another child
            emitter.instruction(&format!("add {scratch}, {scratch}, #1"));      // reserve one conversion level for this value
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("cmp {scratch}, 65"));                 // allow 64 nested arrays and their terminal scalar value
            emitter.instruction(&format!("jae {full}"));                        // reject excessive recursion before packing another child
            emitter.instruction(&format!("add {scratch}, 1"));                  // reserve one conversion level for this value
        }
    }
    abi::emit_store_reg_to_symbol(emitter, scratch, DEPTH, 0);
    abi::emit_call_label(emitter, "__rt_zval_pack_element_impl");
    abi::emit_load_symbol_to_reg(emitter, scratch, DEPTH, 0);
    match emitter.target.arch {
        Arch::AArch64 => emitter.instruction(&format!("sub {scratch}, {scratch}, #1")), // release this completed conversion level without changing the result pointer
        Arch::X86_64 => emitter.instruction(&format!("sub {scratch}, 1")),      // release this completed conversion level without changing the result pointer
    }
    abi::emit_store_reg_to_symbol(emitter, scratch, DEPTH, 0);
    abi::emit_frame_restore(emitter, 16);
    abi::emit_return(emitter);
    emitter.label(full);
    abi::emit_store_imm_to_symbol(emitter, crate::codegen_support::cdylib::BOUNDARY_STATUS, 0,
        crate::codegen_support::cdylib::STATUS_INVALID_ARGUMENT as i64);
    abi::emit_store_zero_to_symbol(emitter, "_exc_value", 0);
    abi::emit_jump(emitter, "__rt_throw_current");
    emitter.label_global("__rt_zval_pack_element_impl");
}

#[cfg(test)]
mod tests {
    use crate::codegen_support::{emit::Emitter, platform::{AppleVariant, Arch, Platform, Target}};

    /// Every library target rejects unsupported tags before normalization can hide pointer identity.
    #[test]
    fn phpext_pack_rejects_non_php_tags_before_normalization() {
        for target in [Target::new(Platform::Linux, Arch::X86_64),
            Target::new(Platform::Linux, Arch::AArch64), Target::new(Platform::MacOS, Arch::AArch64),
            Target::new_apple(Arch::AArch64, AppleVariant::IOS),
            Target::new_apple(Arch::AArch64, AppleVariant::IOSSimulator)] {
            for hosted in [false, true] {
                let mut emitter = Emitter::new_cdylib(target);
                emitter.phpext_output = hosted;
                super::super::zval_pack::emit_zval_pack(&mut emitter);
                let asm = emitter.output();
                if !hosted {
                    assert!(!asm.contains("L_phpext_pack_invalid_result"), "{target:?}");
                    continue;
                }
                let (compare, branch, normalized) = if target.arch == Arch::X86_64 {
                    ("cmp rax, 8", "ja L_phpext_pack_invalid_result", "__rt_zval_pack_element_x86_tag_ready:")
                } else {
                    ("cmp x0, #8", "b.hi L_phpext_pack_invalid_result", "__rt_zval_pack_element_tag_ready:")
                };
                let implementation = asm.find("__rt_zval_pack_element_impl:").unwrap();
                assert!(asm.find(compare).unwrap() < implementation, "{target:?}");
                assert!(asm.find(branch).unwrap() < implementation, "{target:?}");
                assert!(implementation < asm.find(normalized).unwrap(), "{target:?}");
                assert!(asm[..implementation].contains("__rt_throw_current"), "{target:?}");
            }
        }
    }
}
