//! Purpose:
//! Emits string numeric-detection helpers used by PHP loose comparison, `is_numeric()`,
//! the `(float)` string cast, and int-parameter coercion.
//! Converts pointer/length PHP strings through libc `strtod` after clipping them to
//! PHP's own numeric-string grammar.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - The helper returns both a numeric flag and the parsed double without losing PHP byte-string bounds.
//! - `__rt_php_num_scan` runs between `__rt_cstr` and `strtod`, so libc never sees the
//!   spellings PHP's grammar rejects: hexadecimal (`"0x1A"` is `0`, not `26`), `INF` /
//!   `INFINITY` / `NAN` (all `0.0`), and underscore separators. It also owns the
//!   leading/trailing PHP-whitespace rules, so `" 42 "` is numeric while `"1e"` is not.
//! - The numeric flag is PHP's `is_numeric_string(..., allow_errors = 0)`; the parsed
//!   double is always the value of the *leading* numeric run, which is what the
//!   `(float)` cast wants even for `"12abc"`.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_str_to_number`: converts a PHP string to a double and reports whether it is numeric.
///
/// Copies the PHP string into the C-string scratch buffer via `__rt_cstr`, clips it to PHP's
/// leading numeric run with `__rt_php_num_scan`, then parses that run with libc `strtod`.
/// The parsed double is returned in d0/xmm0 (`0.0` when the string has no numeric prefix);
/// the integer result register is 1 when the whole string was numeric (PHP whitespace on
/// either side is allowed), 0 otherwise.
/// Dispatches to the x86_64-specific implementation when targeting that architecture.
pub fn emit_str_to_number(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_str_to_number_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: str_to_number ---");
    emitter.label_global("__rt_str_to_number");

    emitter.instruction("sub sp, sp, #32");                                     // allocate a helper slot for the numeric flag
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // establish a stable helper frame pointer
    emitter.instruction("bl __rt_cstr");                                        // copy the bounded PHP string into the C-string scratch buffer
    emitter.instruction("bl __rt_php_num_scan");                                // clip the scratch to PHP's leading numeric run
    emitter.instruction("str x1, [sp, #0]");                                    // save the fully-numeric flag across strtod
    emitter.instruction("mov x1, #0");                                          // strtod endptr = NULL: the run is already clipped
    emitter.bl_c("strtod");                                                     // parse the clipped numeric run into d0
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the fully-numeric flag as the result

    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the helper stack frame
    emitter.instruction("ret");                                                 // return the numeric flag while preserving the parsed double in d0
}

/// Emits `__rt_str_to_number` for the Linux x86_64 target. Identical logic to the ARM64 path but using
/// x86_64 calling conventions: copies the PHP string via `__rt_cstr`, clips it with
/// `__rt_php_num_scan`, then parses the clipped run with libc `strtod`. Returns 1 in rax when the
/// whole string was numeric, 0 otherwise; the parsed double is preserved in xmm0.
fn emit_str_to_number_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: str_to_number ---");
    emitter.label_global("__rt_str_to_number");

    emitter.instruction("push rbp");                                            // save the caller frame pointer before nested libc calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable helper frame pointer
    emitter.instruction("sub rsp, 32");                                         // allocate an aligned helper slot for the numeric flag
    emitter.instruction("call __rt_cstr");                                      // copy the bounded PHP string into the C-string scratch buffer
    emitter.instruction("mov rdi, rax");                                        // pass the C-string pointer to the numeric-grammar scanner
    emitter.instruction("call __rt_php_num_scan");                              // clip the scratch to PHP's leading numeric run
    emitter.instruction("mov QWORD PTR [rbp - 8], rdx");                        // save the fully-numeric flag across strtod
    emitter.instruction("mov rdi, rax");                                        // pass the clipped numeric run as strtod's first argument
    emitter.instruction("xor esi, esi");                                        // strtod endptr = NULL: the run is already clipped
    emitter.instruction("call strtod");                                         // parse the clipped numeric run into xmm0
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the fully-numeric flag as the result

    emitter.instruction("add rsp, 32");                                         // release the helper stack frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the numeric flag while preserving the parsed double in xmm0
}

/// Emits `__rt_str_looks_like_int_for_coercion` for PHP string-to-int parameter coercion.
///
/// PHP coerces a string into an `int` parameter only when it is a *numeric string*, so this
/// helper is exactly `__rt_php_num_scan`'s fully-numeric flag: leading and trailing PHP
/// whitespace are allowed, while `"0x1A"`, `"INF"`, `"NAN"`, `"1e"` and `"12abc"` are all
/// rejected because the grammar either does not reach them or leaves trailing garbage.
pub fn emit_str_looks_like_int_for_coercion(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_str_looks_like_int_for_coercion_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: str_looks_like_int_for_coercion ---");
    emitter.label_global("__rt_str_looks_like_int_for_coercion");

    emitter.instruction("sub sp, sp, #16");                                     // allocate the minimum aligned helper frame
    emitter.instruction("stp x29, x30, [sp, #0]");                              // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish a stable helper frame pointer
    emitter.instruction("bl __rt_cstr");                                        // copy the bounded PHP string into the C-string scratch buffer
    emitter.instruction("bl __rt_php_num_scan");                                // classify the scratch under PHP's numeric-string grammar
    emitter.instruction("mov x0, x1");                                          // the coercion flag is the fully-numeric flag

    emitter.instruction("ldp x29, x30, [sp, #0]");                              // restore frame pointer and return address
    emitter.instruction("add sp, sp, #16");                                     // release the helper stack frame
    emitter.instruction("ret");                                                 // return the coercion flag
}

/// Emits the Linux x86_64 variant of `__rt_str_looks_like_int_for_coercion`.
///
/// Same contract as the AArch64 helper: returns 1 in `rax` for strings PHP may coerce into
/// an `int` parameter, driven entirely by `__rt_php_num_scan`'s fully-numeric flag.
fn emit_str_looks_like_int_for_coercion_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: str_looks_like_int_for_coercion ---");
    emitter.label_global("__rt_str_looks_like_int_for_coercion");

    emitter.instruction("push rbp");                                            // save the caller frame pointer before nested runtime calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable helper frame pointer
    emitter.instruction("sub rsp, 16");                                         // keep the stack aligned for the nested calls
    emitter.instruction("call __rt_cstr");                                      // copy the bounded PHP string into the C-string scratch buffer
    emitter.instruction("mov rdi, rax");                                        // pass the C-string pointer to the numeric-grammar scanner
    emitter.instruction("call __rt_php_num_scan");                              // classify the scratch under PHP's numeric-string grammar
    emitter.instruction("mov rax, rdx");                                        // the coercion flag is the fully-numeric flag

    emitter.instruction("add rsp, 16");                                         // release the helper stack frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the coercion flag
}
