//! Purpose:
//! Emits the `__rt_ignore_user_abort` runtime helper that holds PHP's global
//! "ignore user abort" flag and implements the get/set semantics of
//! `ignore_user_abort(?bool $enable = null)`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//!
//! Key details:
//! - State lives in `_rt_ignore_user_abort`, whose zero-initialized `.comm` slot is
//!   also the PHP default (0), so no separate seed marker is needed.
//! - The incoming argument arrives in the integer result register. A value equal to
//!   the in-band `NULL_SENTINEL` (the defaulted/explicit PHP null) means "get" (no
//!   change); any other value is coerced to a PHP bool (0/1) and stored. Either way
//!   the *previous* flag (0/1) is returned in the integer result register.

use crate::codegen::sentinels::NULL_SENTINEL;
use crate::codegen::{abi, emit::Emitter, platform::Arch};

/// Emits `__rt_ignore_user_abort` for every supported target.
///
/// Reads the incoming flag from the integer result register (`NULL_SENTINEL`
/// for a null/omitted argument), stores a non-null argument coerced to 0/1 as
/// the new flag, and returns the previous flag (0/1) in the integer result
/// register.
pub fn emit_ignore_user_abort(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ignore_user_abort_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ignore_user_abort ---");
    emitter.label_global("__rt_ignore_user_abort");

    // -- load current flag and decide get vs set --
    abi::emit_load_symbol_to_reg(emitter, "x11", "_rt_ignore_user_abort", 0);   // load the current (soon-to-be previous) ignore-user-abort flag
    abi::emit_load_int_immediate(emitter, "x12", NULL_SENTINEL);                // materialize the in-band null sentinel for the get/set test
    emitter.instruction("cmp x0, x12");                                         // is the incoming argument PHP null (a get, no change)?
    emitter.instruction("b.eq __rt_ignore_user_abort_return");                  // a null argument leaves the flag unchanged
    emitter.instruction("cmp x0, #0");                                          // coerce the incoming value to a PHP bool: is it truthy?
    emitter.instruction("cset x0, ne");                                         // normalize the new flag to 0 or 1
    abi::emit_store_reg_to_symbol(emitter, "x0", "_rt_ignore_user_abort", 0);   // store the normalized flag as the new ignore-user-abort state

    // -- return the previous flag --
    emitter.label("__rt_ignore_user_abort_return");
    emitter.instruction("mov x0, x11");                                         // return the previous flag in the integer result register
    emitter.instruction("ret");                                                 // return to the ignore_user_abort() call site
}

/// Emits `__rt_ignore_user_abort` for x86_64 Linux targets.
///
/// Uses the System V register conventions: the incoming flag and the returned
/// previous flag both live in `rax` (the integer result register). `r8`/`r9`
/// are caller-saved scratch registers safe to clobber in this leaf helper.
fn emit_ignore_user_abort_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ignore_user_abort ---");
    emitter.label_global("__rt_ignore_user_abort");

    // -- load current flag and decide get vs set --
    abi::emit_load_symbol_to_reg(emitter, "r9", "_rt_ignore_user_abort", 0);    // load the current (soon-to-be previous) ignore-user-abort flag
    abi::emit_load_int_immediate(emitter, "r8", NULL_SENTINEL);                 // materialize the in-band null sentinel for the get/set test
    emitter.instruction("cmp rax, r8");                                         // is the incoming argument PHP null (a get, no change)?
    emitter.instruction("je __rt_ignore_user_abort_return");                    // a null argument leaves the flag unchanged
    emitter.instruction("test rax, rax");                                       // coerce the incoming value to a PHP bool: is it truthy?
    emitter.instruction("setne al");                                            // normalize the low byte of the new flag to 0 or 1
    emitter.instruction("movzx rax, al");                                       // zero-extend the normalized flag to the full register
    abi::emit_store_reg_to_symbol(emitter, "rax", "_rt_ignore_user_abort", 0);  // store the normalized flag as the new ignore-user-abort state

    // -- return the previous flag --
    emitter.label("__rt_ignore_user_abort_return");
    emitter.instruction("mov rax, r9");                                         // return the previous flag in the integer result register
    emitter.instruction("ret");                                                 // return to the ignore_user_abort() call site
}
