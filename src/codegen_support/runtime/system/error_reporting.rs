//! Purpose:
//! Emits the `__rt_error_reporting` runtime helper that holds PHP's global
//! error-reporting level and implements the get/set semantics of
//! `error_reporting(?int $error_level = null)`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//!
//! Key details:
//! - State lives in `_rt_error_reporting`; `_rt_error_reporting_init` is a one-shot
//!   seed marker (0 = unseeded → lazily seed the level to E_ALL = 30719) because 0 is
//!   itself a valid reporting level and cannot double as "unseeded".
//! - The incoming argument arrives in the integer result register. A value equal to
//!   the in-band `NULL_SENTINEL` (the defaulted/explicit PHP null) means "get" (no
//!   change); any other value is the new level to store. Either way the *previous*
//!   level is returned in the integer result register, matching PHP.

use crate::codegen::sentinels::NULL_SENTINEL;
use crate::codegen::{abi, emit::Emitter, platform::Arch};

/// E_ALL as of the supported PHP version: the seed value stored into
/// `_rt_error_reporting` on the first access when it is still unseeded.
const E_ALL_SEED: i64 = 30719;

/// Emits `__rt_error_reporting` for every supported target.
///
/// Reads the incoming level from the integer result register (`NULL_SENTINEL`
/// for a null/omitted argument), lazily seeds the global level to `E_ALL`
/// (30719) on first use, stores a non-null argument as the new level, and
/// returns the previous level in the integer result register.
pub fn emit_error_reporting(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_error_reporting_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: error_reporting ---");
    emitter.label_global("__rt_error_reporting");

    // -- lazily seed the level to E_ALL on first access --
    abi::emit_load_symbol_to_reg(emitter, "x10", "_rt_error_reporting_init", 0); // load the one-shot seed marker (0 = unseeded)
    emitter.instruction("cbnz x10, __rt_error_reporting_seeded");               // skip seeding once the level has been initialized
    abi::emit_load_int_immediate(emitter, "x11", E_ALL_SEED);                   // materialize the E_ALL default reporting level
    abi::emit_store_reg_to_symbol(emitter, "x11", "_rt_error_reporting", 0);    // seed the global reporting level with E_ALL
    emitter.instruction("mov x10, #1");                                         // mark the reporting level as seeded
    abi::emit_store_reg_to_symbol(emitter, "x10", "_rt_error_reporting_init", 0); // publish the seed marker so seeding runs only once
    emitter.label("__rt_error_reporting_seeded");

    // -- load current level and decide get vs set --
    abi::emit_load_symbol_to_reg(emitter, "x11", "_rt_error_reporting", 0);     // load the current (soon-to-be previous) reporting level
    abi::emit_load_int_immediate(emitter, "x12", NULL_SENTINEL);                // materialize the in-band null sentinel for the get/set test
    emitter.instruction("cmp x0, x12");                                         // is the incoming argument PHP null (a get, no change)?
    emitter.instruction("b.eq __rt_error_reporting_return");                    // a null argument leaves the level unchanged
    abi::emit_store_reg_to_symbol(emitter, "x0", "_rt_error_reporting", 0);     // store the requested level as the new reporting level

    // -- return the previous level --
    emitter.label("__rt_error_reporting_return");
    emitter.instruction("mov x0, x11");                                         // return the previous reporting level in the integer result register
    emitter.instruction("ret");                                                 // return to the error_reporting() call site
}

/// Emits `__rt_error_reporting` for x86_64 Linux targets.
///
/// Uses the System V register conventions: the incoming level and the returned
/// previous level both live in `rax` (the integer result register). Globals are
/// accessed through the shared PIC-safe symbol load/store helpers; `r8`/`r9`
/// are caller-saved scratch registers safe to clobber in this leaf helper.
fn emit_error_reporting_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: error_reporting ---");
    emitter.label_global("__rt_error_reporting");

    // -- lazily seed the level to E_ALL on first access --
    abi::emit_load_symbol_to_reg(emitter, "r8", "_rt_error_reporting_init", 0); // load the one-shot seed marker (0 = unseeded)
    emitter.instruction("test r8, r8");                                         // has the reporting level already been seeded?
    emitter.instruction("jnz __rt_error_reporting_seeded");                     // skip seeding once the level has been initialized
    abi::emit_load_int_immediate(emitter, "r9", E_ALL_SEED);                    // materialize the E_ALL default reporting level
    abi::emit_store_reg_to_symbol(emitter, "r9", "_rt_error_reporting", 0);     // seed the global reporting level with E_ALL
    emitter.instruction("mov r8, 1");                                           // mark the reporting level as seeded
    abi::emit_store_reg_to_symbol(emitter, "r8", "_rt_error_reporting_init", 0); // publish the seed marker so seeding runs only once
    emitter.label("__rt_error_reporting_seeded");

    // -- load current level and decide get vs set --
    abi::emit_load_symbol_to_reg(emitter, "r9", "_rt_error_reporting", 0);      // load the current (soon-to-be previous) reporting level
    abi::emit_load_int_immediate(emitter, "r8", NULL_SENTINEL);                 // materialize the in-band null sentinel for the get/set test
    emitter.instruction("cmp rax, r8");                                         // is the incoming argument PHP null (a get, no change)?
    emitter.instruction("je __rt_error_reporting_return");                      // a null argument leaves the level unchanged
    abi::emit_store_reg_to_symbol(emitter, "rax", "_rt_error_reporting", 0);    // store the requested level as the new reporting level

    // -- return the previous level --
    emitter.label("__rt_error_reporting_return");
    emitter.instruction("mov rax, r9");                                         // return the previous reporting level in the integer result register
    emitter.instruction("ret");                                                 // return to the error_reporting() call site
}
