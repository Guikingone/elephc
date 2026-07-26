//! Purpose:
//! Emits the `__rt_warn_foreach_non_iterable` runtime helper: PHP's
//! `foreach() argument must be of type array|object, <type> given` E_WARNING.
//! Also owns the message table so the codegen emitter and the `.data` emitter agree
//! on both the byte contents and the byte lengths of every variant.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//! - `crate::codegen_support::runtime::data::fixed` for the string literals themselves.
//!
//! Key details:
//! - PHP names the OFFENDING VALUE, not its declared type: a `bool` prints `true`/`false`,
//!   never `bool`. The helper therefore takes the payload low word alongside the tag and
//!   picks the literal at runtime.
//! - Each variant is ONE complete message rather than prefix + name + suffix, so the helper
//!   needs a single `__rt_diag_warning` call and never touches the `_concat_buf` scratch
//!   (unlike `__rt_warn_undefined_array_key_int`, which must save/restore `_concat_off`).
//! - The helper RETURNS. It replaced `__rt_iterable_unsupported_kind` on the foreach paths,
//!   which wrote a compiler-internal fatal and exited 70; PHP warns and skips the loop.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// The `foreach()` warning text for a value the runtime cannot iterate.
///
/// Entries are `(symbol, message)`. `crate::codegen_support::runtime::data::fixed`
/// emits them verbatim as `.ascii` literals; this module derives each `write()`
/// length from `message.len()` so the two can never drift.
///
/// The wording is php-src's `zend_type_error`-shaped `E_WARNING` from
/// `ZEND_FE_RESET_R`, captured from PHP 8.5.6 with `php -d xdebug.mode=off`:
/// `foreach() argument must be of type array|object, <type> given`. elephc does not
/// synthesize the ` in <file> on line <n>` tail that php-src appends.
pub const FOREACH_NON_ITERABLE_MESSAGES: &[(&str, &str)] = &[
    (
        "_diag_foreach_arg_int",
        "Warning: foreach() argument must be of type array|object, int given\n",
    ),
    (
        "_diag_foreach_arg_string",
        "Warning: foreach() argument must be of type array|object, string given\n",
    ),
    (
        "_diag_foreach_arg_float",
        "Warning: foreach() argument must be of type array|object, float given\n",
    ),
    (
        "_diag_foreach_arg_true",
        "Warning: foreach() argument must be of type array|object, true given\n",
    ),
    (
        "_diag_foreach_arg_false",
        "Warning: foreach() argument must be of type array|object, false given\n",
    ),
    (
        "_diag_foreach_arg_null",
        "Warning: foreach() argument must be of type array|object, null given\n",
    ),
    (
        "_diag_foreach_arg_resource",
        "Warning: foreach() argument must be of type array|object, resource given\n",
    ),
];

/// Returns the byte length of the message stored under `symbol`.
///
/// Panics when the symbol is not in `FOREACH_NON_ITERABLE_MESSAGES`, which would mean the
/// emitter and the table disagree — a compiler bug, not a user-reachable condition.
fn message_len(symbol: &str) -> usize {
    FOREACH_NON_ITERABLE_MESSAGES
        .iter()
        .find(|(name, _)| *name == symbol)
        .map(|(_, message)| message.len())
        .unwrap_or_else(|| panic!("unknown foreach warning symbol {symbol}"))
}

/// Emits the `__rt_warn_foreach_non_iterable` runtime helper.
///
/// Dispatches to the target-specific implementation; x86_64 uses the System V
/// register convention, every other target uses the AArch64 path.
///
/// # ABI
/// Input: `x0` / `rax` = runtime value tag (`crate::codegen_support::value_boxing::runtime_value_tag`
/// scheme: 0 int, 1 string, 2 float, 3 bool, 8 null, 9 resource). `x1` / `rdi` = payload low
/// word, read ONLY for tag 3 to choose between `true` and `false`.
/// Output: none. Clobbers the caller-saved scratch that `__rt_diag_warning` clobbers.
/// Returns normally — callers resume with an empty iterator state so the loop body is skipped.
///
/// Tags outside the handled set (7 `Mixed`, 10 `Callable`, and the array/object tags 4/5/6,
/// which never reach a caller) fall through to the `null` message. `Mixed` cannot survive
/// `__rt_mixed_unbox`, and the array/object tags are dispatched away before the call.
pub fn emit_foreach_non_iterable_warning(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_foreach_non_iterable_warning_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: foreach_non_iterable_warning ---");
    emitter.label_global("__rt_warn_foreach_non_iterable");
    emitter.instruction("stp x29, x30, [sp, #-16]!");                           // preserve frame linkage across the diagnostic call
    emitter.instruction("mov x29, sp");                                         // establish a stable warning helper frame
    emitter.instruction("mov x9, x1");                                          // preserve the payload low word before x1 becomes the message pointer

    // -- select the message that names the offending VALUE, PHP-style --
    emitter.instruction("cmp x0, #0");                                          // runtime tag 0 = int
    emitter.instruction("b.eq __rt_warn_foreach_non_iterable_int");             // report the int-valued foreach argument
    emitter.instruction("cmp x0, #1");                                          // runtime tag 1 = string
    emitter.instruction("b.eq __rt_warn_foreach_non_iterable_string");          // report the string-valued foreach argument
    emitter.instruction("cmp x0, #2");                                          // runtime tag 2 = float
    emitter.instruction("b.eq __rt_warn_foreach_non_iterable_float");           // report the float-valued foreach argument
    emitter.instruction("cmp x0, #3");                                          // runtime tag 3 = bool
    emitter.instruction("b.eq __rt_warn_foreach_non_iterable_bool");            // choose true/false from the preserved payload
    emitter.instruction("cmp x0, #9");                                          // runtime tag 9 = resource
    emitter.instruction("b.eq __rt_warn_foreach_non_iterable_resource");        // report the resource-valued foreach argument
    emitter.instruction("b __rt_warn_foreach_non_iterable_null");               // tag 8 and every unmapped tag report null

    emitter.label("__rt_warn_foreach_non_iterable_bool");
    emitter.instruction("cbz x9, __rt_warn_foreach_non_iterable_false");        // a zero bool payload is PHP's false literal
    emit_message_selection_aarch64(emitter, "_diag_foreach_arg_true");
    emitter.label("__rt_warn_foreach_non_iterable_false");
    emit_message_selection_aarch64(emitter, "_diag_foreach_arg_false");
    emitter.label("__rt_warn_foreach_non_iterable_int");
    emit_message_selection_aarch64(emitter, "_diag_foreach_arg_int");
    emitter.label("__rt_warn_foreach_non_iterable_string");
    emit_message_selection_aarch64(emitter, "_diag_foreach_arg_string");
    emitter.label("__rt_warn_foreach_non_iterable_float");
    emit_message_selection_aarch64(emitter, "_diag_foreach_arg_float");
    emitter.label("__rt_warn_foreach_non_iterable_resource");
    emit_message_selection_aarch64(emitter, "_diag_foreach_arg_resource");
    emitter.label("__rt_warn_foreach_non_iterable_null");
    emit_message_selection_aarch64(emitter, "_diag_foreach_arg_null");

    emitter.label("__rt_warn_foreach_non_iterable_emit");
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the PHP foreach warning
    emitter.instruction("ldp x29, x30, [sp], #16");                             // restore frame linkage
    emitter.instruction("ret");                                                 // return so the caller can skip the loop body
}

/// Points `x1`/`x2` at one complete foreach warning and jumps to the shared emit tail.
fn emit_message_selection_aarch64(emitter: &mut Emitter, symbol: &str) {
    abi::emit_symbol_address(emitter, "x1", symbol);
    emitter.instruction(&format!("mov x2, #{}", message_len(symbol)));          // pass the complete foreach warning length
    emitter.instruction("b __rt_warn_foreach_non_iterable_emit");               // share one diagnostic call across every variant
}

/// x86_64 implementation of `__rt_warn_foreach_non_iterable`.
///
/// `rax` carries the runtime value tag and `rdi` the payload low word, matching what
/// `__rt_mixed_unbox` leaves behind. The payload is parked in `r11` (caller-saved, and
/// untouched by `__rt_diag_warning`, which uses `rax`/`rdx`/`rsi`/`rdi`/`r10`) because
/// `rdi` doubles as the System V message-pointer argument.
fn emit_foreach_non_iterable_warning_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: foreach_non_iterable_warning ---");
    emitter.label_global("__rt_warn_foreach_non_iterable");
    emitter.instruction("push rbp");                                            // preserve the caller frame and align the diagnostic call
    emitter.instruction("mov rbp, rsp");                                        // establish a stable warning helper frame
    emitter.instruction("mov r11, rdi");                                        // preserve the payload low word before rdi becomes the message pointer

    // -- select the message that names the offending VALUE, PHP-style --
    emitter.instruction("cmp rax, 0");                                          // runtime tag 0 = int
    emitter.instruction("je __rt_warn_foreach_non_iterable_int");               // report the int-valued foreach argument
    emitter.instruction("cmp rax, 1");                                          // runtime tag 1 = string
    emitter.instruction("je __rt_warn_foreach_non_iterable_string");            // report the string-valued foreach argument
    emitter.instruction("cmp rax, 2");                                          // runtime tag 2 = float
    emitter.instruction("je __rt_warn_foreach_non_iterable_float");             // report the float-valued foreach argument
    emitter.instruction("cmp rax, 3");                                          // runtime tag 3 = bool
    emitter.instruction("je __rt_warn_foreach_non_iterable_bool");              // choose true/false from the preserved payload
    emitter.instruction("cmp rax, 9");                                          // runtime tag 9 = resource
    emitter.instruction("je __rt_warn_foreach_non_iterable_resource");          // report the resource-valued foreach argument
    emitter.instruction("jmp __rt_warn_foreach_non_iterable_null");             // tag 8 and every unmapped tag report null

    emitter.label("__rt_warn_foreach_non_iterable_bool");
    emitter.instruction("test r11, r11");                                       // is the preserved bool payload zero?
    emitter.instruction("jz __rt_warn_foreach_non_iterable_false");             // a zero bool payload is PHP's false literal
    emit_message_selection_x86_64(emitter, "_diag_foreach_arg_true");
    emitter.label("__rt_warn_foreach_non_iterable_false");
    emit_message_selection_x86_64(emitter, "_diag_foreach_arg_false");
    emitter.label("__rt_warn_foreach_non_iterable_int");
    emit_message_selection_x86_64(emitter, "_diag_foreach_arg_int");
    emitter.label("__rt_warn_foreach_non_iterable_string");
    emit_message_selection_x86_64(emitter, "_diag_foreach_arg_string");
    emitter.label("__rt_warn_foreach_non_iterable_float");
    emit_message_selection_x86_64(emitter, "_diag_foreach_arg_float");
    emitter.label("__rt_warn_foreach_non_iterable_resource");
    emit_message_selection_x86_64(emitter, "_diag_foreach_arg_resource");
    emitter.label("__rt_warn_foreach_non_iterable_null");
    emit_message_selection_x86_64(emitter, "_diag_foreach_arg_null");

    emitter.label("__rt_warn_foreach_non_iterable_emit");
    abi::emit_call_label(emitter, "__rt_diag_warning");                         // emit or suppress the PHP foreach warning
    emitter.instruction("mov rsp, rbp");                                        // release the warning helper frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return so the caller can skip the loop body
}

/// Points `rdi`/`esi` at one complete foreach warning and jumps to the shared emit tail.
fn emit_message_selection_x86_64(emitter: &mut Emitter, symbol: &str) {
    abi::emit_symbol_address(emitter, "rdi", symbol);
    emitter.instruction(&format!("mov esi, {}", message_len(symbol)));          // pass the complete foreach warning length
    emitter.instruction("jmp __rt_warn_foreach_non_iterable_emit");             // share one diagnostic call across every variant
}
