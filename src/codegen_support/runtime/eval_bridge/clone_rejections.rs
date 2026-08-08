//! Purpose:
//! Rejects runtime-managed object payloads from generic clone paths.
//!
//! Called from:
//! - The eval bridge runtime facade and sibling bridge emitters.
//!
//! Key details:
//! - The architecture-specific class-id comparisons remain symmetric.

use super::*;

/// Emits ARM64 comparisons that keep runtime-managed object payloads out of the generic clone path.
pub(super) fn emit_aarch64_reject_runtime_managed_clone_classes(
    emitter: &mut Emitter,
    class_id_reg: &str,
    reject_label: &str,
) {
    for symbol in [
        "_fiber_class_id",
        "_generator_class_id",
        "_spl_dll_class_id",
        "_spl_stack_class_id",
        "_spl_queue_class_id",
        "_spl_fixed_array_class_id",
    ] {
        abi::emit_symbol_address(emitter, "x10", symbol);
        emitter.instruction("ldr x10, [x10]");                                  // load one runtime-managed class id sentinel
        emitter.instruction(&format!("cmp {}, x10", class_id_reg));             // compare source class id with the unsupported payload sentinel
        emitter.instruction(&format!("b.eq {}", reject_label));                 // reject custom runtime payload layouts
    }
}

/// Emits x86_64 comparisons that keep runtime-managed object payloads out of the generic clone path.
pub(super) fn emit_x86_64_reject_runtime_managed_clone_classes(
    emitter: &mut Emitter,
    class_id_reg: &str,
    reject_label: &str,
) {
    for symbol in [
        "_fiber_class_id",
        "_generator_class_id",
        "_spl_dll_class_id",
        "_spl_stack_class_id",
        "_spl_queue_class_id",
        "_spl_fixed_array_class_id",
    ] {
        abi::emit_load_symbol_to_reg(emitter, "r11", symbol, 0);
        emitter.instruction(&format!("cmp {}, r11", class_id_reg));             // compare source class id with the unsupported payload sentinel
        emitter.instruction(&format!("je {}", reject_label));                   // reject custom runtime payload layouts
    }
}
