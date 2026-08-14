//! Purpose:
//! Publishes the elephc-bcmath C ABI into late-bound runtime function-pointer slots.
//!
//! Called from:
//! - BCMath EIR lowerers immediately before invoking a shared `__rt_bc*` helper.
//!
//! Key details:
//! - Call-site publication keeps programs without BCMath calls free of bridge symbol references.
//! - Every C entry is published for both AArch64 and x86_64 supported targets.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Publishes every elephc-bcmath entry point into its shared runtime slot.
pub(crate) fn publish_elephc_bcmath_function_pointers(emitter: &mut Emitter) {
    const ENTRIES: &[(&str, &str)] = &[
        ("elephc_bcmath_add", "_elephc_bcmath_add_fn"),
        ("elephc_bcmath_sub", "_elephc_bcmath_sub_fn"),
        ("elephc_bcmath_mul", "_elephc_bcmath_mul_fn"),
        ("elephc_bcmath_div", "_elephc_bcmath_div_fn"),
        ("elephc_bcmath_mod", "_elephc_bcmath_mod_fn"),
        ("elephc_bcmath_divmod", "_elephc_bcmath_divmod_fn"),
        ("elephc_bcmath_pow", "_elephc_bcmath_pow_fn"),
        ("elephc_bcmath_powmod", "_elephc_bcmath_powmod_fn"),
        ("elephc_bcmath_sqrt", "_elephc_bcmath_sqrt_fn"),
        ("elephc_bcmath_comp", "_elephc_bcmath_comp_fn"),
        ("elephc_bcmath_get_scale", "_elephc_bcmath_get_scale_fn"),
        ("elephc_bcmath_set_scale", "_elephc_bcmath_set_scale_fn"),
        ("elephc_bcmath_ceil", "_elephc_bcmath_ceil_fn"),
        ("elephc_bcmath_floor", "_elephc_bcmath_floor_fn"),
        ("elephc_bcmath_round", "_elephc_bcmath_round_fn"),
        (
            "elephc_bcmath_last_error",
            "_elephc_bcmath_last_error_fn",
        ),
        ("elephc_bcmath_free", "_elephc_bcmath_free_fn"),
    ];
    match emitter.target.arch {
        Arch::AArch64 => {
            for (c_name, slot) in ENTRIES {
                let extern_symbol = emitter.target.extern_symbol(c_name);
                abi::emit_extern_symbol_address(emitter, "x9", &extern_symbol);
                abi::emit_symbol_address(emitter, "x10", slot);
                emitter.instruction("str x9, [x10]");                           // publish one elephc-bcmath entry into its late-bound runtime slot
            }
        }
        Arch::X86_64 => {
            for (c_name, slot) in ENTRIES {
                let extern_symbol = emitter.target.extern_symbol(c_name);
                abi::emit_extern_symbol_address(emitter, "r9", &extern_symbol);
                abi::emit_store_reg_to_symbol(emitter, "r9", slot, 0);         // publish one elephc-bcmath entry into its late-bound runtime slot
            }
        }
    }
}
