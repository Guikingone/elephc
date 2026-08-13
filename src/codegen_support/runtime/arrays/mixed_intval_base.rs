//! Purpose:
//! Emits the `__rt_mixed_intval_base` runtime helper: PHP `intval($value, $base)` applied to a
//! boxed `Mixed` cell whose payload type is only known at run time.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//!
//! Key details:
//! - php-src's `PHP_FUNCTION(intval)` honors `$base` only when the subject is a string and
//!   otherwise behaves exactly like a plain `(int)` cast, so this helper unboxes the cell,
//!   routes a tag-1 string payload to `__rt_str_to_int_base`, and hands every other payload
//!   to `__rt_mixed_cast_int` unchanged.
//! - The boxed pointer is saved before the unbox because the fallback path needs the original
//!   cell, and the base is saved because `__rt_mixed_unbox` owns the argument registers.
//! - Mixed helpers use boxed tag/payload cells; tag constants and ownership rules are shared
//!   with type checking and codegen.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_mixed_intval_base` runtime helper.
///
/// ABI (AArch64):
///   Input:  `x0` = boxed mixed pointer, `x3` = requested base.
///   Output: `x0` = the PHP integer value.
///
/// ABI (x86_64 System V):
///   Input:  `rax` = boxed mixed pointer, `rcx` = requested base.
///   Output: `rax` = the PHP integer value.
///
/// The input registers mirror `__rt_mixed_cast_int`, which this helper tail-calls for every
/// payload PHP's `$base` does not apply to.
pub fn emit_mixed_intval_base(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mixed_intval_base_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: mixed_intval_base ---");
    emitter.label_global("__rt_mixed_intval_base");

    emitter.instruction("sub sp, sp, #32");                                     // allocate a small stack frame for the nested helper calls
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // establish the helper stack frame
    emitter.instruction("str x0, [sp]");                                        // keep the boxed pointer for the non-string fallback
    emitter.instruction("str x3, [sp, #8]");                                    // keep the requested base across the unbox call
    emitter.instruction("bl __rt_mixed_unbox");                                 // x0=tag, x1=value_lo, x2=value_hi for the boxed payload
    emitter.instruction("cmp x0, #1");                                          // does the mixed payload hold a string?
    emitter.instruction("b.ne __rt_mixed_intval_base_cast");                    // every other payload ignores PHP's $base argument
    emitter.instruction("ldr x3, [sp, #8]");                                    // restore the requested base for the string parser
    emitter.instruction("bl __rt_str_to_int_base");                             // parse the unboxed string payload in the requested base
    emitter.instruction("b __rt_mixed_intval_base_done");                       // return the parsed integer result

    emitter.label("__rt_mixed_intval_base_cast");
    emitter.instruction("ldr x0, [sp]");                                        // restore the boxed pointer for the ordinary integer cast
    emitter.instruction("bl __rt_mixed_cast_int");                              // non-string payloads cast exactly like one-argument intval()

    emitter.label("__rt_mixed_intval_base_done");
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the helper stack frame
    emitter.instruction("ret");                                                 // return the integer result in x0
}

/// Emits the x86_64 Linux variant of `__rt_mixed_intval_base`.
///
/// `__rt_mixed_unbox` returns the tag in `rax` and the payload words in `rdi`/`rdx` here, so a
/// string payload already has its pointer in `rdi` and only needs its length moved into `rsi`
/// before the base parser's System V argument list is complete.
///
/// # ABI
/// - Input: rax = boxed mixed pointer, rcx = requested base
/// - Output: rax = integer result
/// - Clobbers: rax, rcx, rdi, rsi, rdx, xmm0, rsp; preserves rbp
fn emit_mixed_intval_base_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_intval_base ---");
    emitter.label_global("__rt_mixed_intval_base");

    emitter.instruction("push rbp");                                            // save the caller frame pointer before this helper allocates its own frame
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame pointer for the helper body
    emitter.instruction("sub rsp, 16");                                         // reserve one aligned temporary slot so nested helper calls keep the SysV stack aligned
    emitter.instruction("mov QWORD PTR [rsp], rax");                            // keep the boxed pointer for the non-string fallback
    emitter.instruction("mov QWORD PTR [rsp + 8], rcx");                        // keep the requested base across the unbox call
    abi::emit_call_label(emitter, "__rt_mixed_unbox");                          // return the mixed runtime tag in rax and payload words in rdi/rdx
    emitter.instruction("cmp rax, 1");                                          // does the mixed payload hold a string?
    emitter.instruction("jne __rt_mixed_intval_base_cast_linux_x86_64");        // every other payload ignores PHP's $base argument
    emitter.instruction("mov rsi, rdx");                                        // move the unboxed string length into the parser's second argument
    emitter.instruction("mov rdx, QWORD PTR [rsp + 8]");                        // restore the requested base for the string parser
    abi::emit_call_label(emitter, "__rt_str_to_int_base");                      // parse the unboxed string payload in the requested base
    emitter.instruction("jmp __rt_mixed_intval_base_done_linux_x86_64");        // return the parsed integer result

    emitter.label("__rt_mixed_intval_base_cast_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rsp]");                            // restore the boxed pointer for the ordinary integer cast
    abi::emit_call_label(emitter, "__rt_mixed_cast_int");                       // non-string payloads cast exactly like one-argument intval()

    emitter.label("__rt_mixed_intval_base_done_linux_x86_64");
    emitter.instruction("add rsp, 16");                                         // release the aligned temporary slot reserved for nested helper calls
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning
    emitter.instruction("ret");                                                 // return the integer result in rax
}
