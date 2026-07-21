//! Purpose:
//! Emits the `__rt_exception_construct` runtime helper: the standalone method body
//! for the built-in Throwable roots' `__construct` (`Exception`/`Error`).
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::exceptions`.
//! - Reached at run time through the `_method_<Root>___construct` wrapper symbols that
//!   `crate::codegen::emit_intrinsic_method_wrappers()` emits for referenced roots.
//!
//! Key details:
//! - The helper is a no-frame leaf: it only stores its incoming method-ABI arguments
//!   into the compact Throwable payload, then returns.
//! - Payload field offsets (`msg_ptr@8`, `msg_len@16`, `code@24`) and the borrow-store
//!   contract must stay identical to `lower_builtin_throwable_new` (the inline `new`
//!   path) and to `lower_throwable_get_message` / `lower_throwable_get_code` (the readers).

use crate::codegen::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits `__rt_exception_construct` for both supported target architectures.
///
/// Implements the built-in Throwable roots' `__construct(string $message, int $code,
/// ?Throwable $previous)`. The method ABI delivers `$this` (the object payload pointer)
/// in integer argument register 0, the `$message` string in the register pair 1/2
/// (pointer, length), `$code` in register 3, and the `?Throwable $previous` box in
/// register 4. The helper writes the message pointer/length and code into the compact
/// Throwable payload at offsets 8/16/24, mirroring `lower_builtin_throwable_new` and the
/// `getMessage`/`getCode` readers. The message pointer/length are stored as-is (no retain
/// or copy), matching the inline `new` field-store contract.
///
/// `$previous` is not retained by the compact payload, but the method-call ABI always
/// materializes it as an *owned* boxed Mixed temporary (even a null default is boxed).
/// Because the compact payload has no slot for it and the caller does not release it, the
/// helper releases that owned temp through `__rt_decref_any`, exactly as a normal
/// EIR-lowered method epilogue would release an owned Mixed parameter. The release is a
/// tail-call: the leaf did no `bl`/`call` before it, so the link register is intact and
/// `__rt_decref_any` returns straight to the constructor's caller (whose discarded result
/// register the caller overwrites with the void sentinel).
pub fn emit_exception_construct(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: exception_construct ---");
    emitter.label_global("__rt_exception_construct");
    // Read the method-ABI arguments: reg0 = $this, reg1/reg2 = $message ptr/len, reg3 = $code.
    let this_reg = abi::int_arg_reg_name(emitter.target, 0);
    let message_ptr_reg = abi::int_arg_reg_name(emitter.target, 1);
    let message_len_reg = abi::int_arg_reg_name(emitter.target, 2);
    let code_reg = abi::int_arg_reg_name(emitter.target, 3);
    let previous_reg = abi::int_arg_reg_name(emitter.target, 4);
    // Store into the compact Throwable payload; offsets mirror the inline new/get paths.
    abi::emit_store_to_address(emitter, message_ptr_reg, this_reg, 8);
    abi::emit_store_to_address(emitter, message_len_reg, this_reg, 16);
    abi::emit_store_to_address(emitter, code_reg, this_reg, 24);
    // Release the owned $previous box and tail-return through the release helper.
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("mov x0, {}", previous_reg));          // pass the owned $previous box to the release helper
            emitter.instruction("b __rt_decref_any");                           // release it and tail-return to the constructor's caller
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("mov rax, {}", previous_reg));         // pass the owned $previous box to the release helper
            emitter.instruction("jmp __rt_decref_any");                         // release it and tail-return to the constructor's caller
        }
    }
}
