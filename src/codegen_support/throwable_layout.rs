//! Purpose: share declared Throwable storage between native allocation and field access.
//! Called from: EIR object lowering and runtime-generated exception paths.
//! Key details: these are Elephc property slots, not Zend's in-memory zval layout.

use super::{abi, emit::Emitter, platform::Arch};

// Keep in step with the inherited fields in builtin_types::declarations. Each
// declared property occupies sixteen bytes after the eight-byte class id.
pub(crate) const FILE_OFFSET: usize = 8 + 3 * 16;
pub(crate) const LINE_OFFSET: usize = 8 + 4 * 16;
pub(crate) const PREVIOUS_OFFSET: usize = 8 + 6 * 16;
pub(crate) const PAYLOAD_SIZE: usize = 8 + 7 * 16;

/// Allocates and clears every declared slot before any property or GC walk.
/// The caller still stamps the class id and heap kind and acquires its handle.
/// User subclasses must pass their actual layout size, including any dynamic tail.
pub(crate) fn emit_allocate(emitter: &mut Emitter, payload_size: usize) {
    assert!(payload_size >= PAYLOAD_SIZE);
    let result = abi::int_result_reg(emitter);
    abi::emit_load_int_immediate(emitter, result, payload_size as i64);
    abi::emit_call_label(emitter, "__rt_heap_alloc");
    // -- recycled heap blocks must not expose stale owned property pointers --
    for offset in (8..payload_size).step_by(8) {
        match emitter.target.arch {
            Arch::AArch64 => emitter.instruction(&format!("str xzr, [x0, #{}]", offset)), // initialize the complete declared property region
            Arch::X86_64 => emitter.instruction(&format!("mov QWORD PTR [rax + {}], 0", offset)), // initialize the complete declared property region
        }
    }
    emit_initial_file(emitter, result);
}

/// Initializes the per-object source-file property with the known module path.
/// This does not invent missing include/stack provenance; callers with a more
/// precise source location can subsequently replace the ordinary property.
pub(crate) fn emit_initial_file(emitter: &mut Emitter, object: &str) {
    let scratch = match emitter.target.arch {
        Arch::AArch64 => "x10",
        Arch::X86_64 => "r10",
    };
    abi::emit_symbol_address(emitter, scratch, "_script_source_file");
    abi::emit_store_to_address(emitter, scratch, object, FILE_OFFSET);
    abi::emit_load_symbol_to_reg(emitter, scratch, "_script_source_file_len", 0);
    abi::emit_store_to_address(emitter, scratch, object, FILE_OFFSET + 8);
}

/// Replaces an initialized default file without leaking its owned string.
pub(crate) fn emit_replace_initial_file(emitter: &mut Emitter, object: &str) {
    abi::emit_push_reg(emitter, object);
    let result = abi::int_result_reg(emitter);
    abi::emit_load_from_address(emitter, result, object, FILE_OFFSET);
    abi::emit_call_label(emitter, "__rt_decref_any");
    abi::emit_pop_reg(emitter, object);
    emit_initial_file(emitter, object);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::Target;

    #[test]
    fn throwable_layout_matches_declared_builtin_properties() {
        let tokens = crate::lexer::tokenize("<?php new Exception(); new Error(); new ReflectionException();").unwrap();
        let program = crate::parser::parse(&tokens).unwrap();
        let checked = crate::types::check(&program).unwrap();
        for name in ["Exception", "Error", "ReflectionException"] {
            let info = &checked.classes[name];
            assert_eq!(8 + info.properties.len() * 16, PAYLOAD_SIZE, "{name}");
            for (property, expected) in [("file", FILE_OFFSET), ("line", LINE_OFFSET), ("previous", PREVIOUS_OFFSET)] {
                let index = info.properties.iter().position(|(name, _)| name == property).unwrap();
                assert_eq!(8 + index * 16, expected, "{name}::{property}");
            }
        }
    }

    #[test]
    fn throwable_layout_allocation_initializes_all_supported_targets() {
        for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
            let target = Target::parse(name).unwrap();
            let mut emitter = Emitter::new(target);
            emit_allocate(&mut emitter, PAYLOAD_SIZE);
            let asm = emitter.output();
            match target.arch {
                Arch::AArch64 => {
                    assert!(asm.contains("mov x0, #120"), "{name}: {asm}");
                    for offset in (8..PAYLOAD_SIZE).step_by(8) {
                        assert!(asm.contains(&format!("str xzr, [x0, #{offset}]")), "{name}: {asm}");
                    }
                    assert!(asm.contains("str x10, [x0, #56]"), "{name}: {asm}");
                    assert!(asm.contains("str x10, [x0, #64]"), "{name}: {asm}");
                }
                Arch::X86_64 => {
                    assert!(asm.contains("mov rax, 120"), "{name}: {asm}");
                    for offset in (8..PAYLOAD_SIZE).step_by(8) {
                        assert!(asm.contains(&format!("mov QWORD PTR [rax + {offset}], 0")), "{name}: {asm}");
                    }
                    assert!(asm.contains("mov QWORD PTR [rax + 56], r10"), "{name}: {asm}");
                    assert!(asm.contains("mov QWORD PTR [rax + 64], r10"), "{name}: {asm}");
                }
            }
        }
    }
}
