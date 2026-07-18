//! Purpose:
//! Emits the `__rt_pr_cap_*` runtime walkers that render PHP `print_r($v, true)`
//! output into an owned heap string instead of writing to stdout. A SEPARATE,
//! self-contained family from `__rt_print_r_*` (`print_r_walk.rs`) rather than a
//! shared dual-mode walker: the stdout walkers write immediately via syscalls
//! sprinkled through deeply nested control flow, so threading a capture-buffer
//! cursor through every call site would have meant rewriting that file anyway;
//! duplicating the (smaller, append-only) control flow here keeps both variants
//! independently readable.
//!
//! Called from:
//! - `crate::codegen::runtime::emitters::emit_runtime()` via `crate::codegen::runtime::io`.
//! - `crate::codegen_ir::lower_inst::builtins::debug` (the `print_r($v, true)` EIR lowering)
//!   for scalar/array/hash/Mixed values.
//!
//! Key details:
//! - Output format matches `__rt_print_r_*` exactly (same `Array\n(\n    [k] => v\n)\n`
//!   layout, same scalar rendering: bool `true`→`"1"`, `false`/`null`→`""`).
//! - Bytes accumulate in the fixed 1 MiB `_pr_cap_buf` scratch region at cursor
//!   `_pr_cap_off`, reset to 0 by the caller (`lower_print_r`) before the first
//!   append of a top-level `print_r($v, true)` call. Exceeding the 1 MiB cap is a
//!   loud runtime fatal (`_pr_cap_overflow_msg`), never silent truncation.
//! - `_pr_cap_depth` bounds array/hash nesting at `MAX_DEPTH` (256, matching
//!   common debugger nesting defaults). This is NOT true cycle detection — it is
//!   the JURY ADDENDUM's alternative "scope out with a loud runtime fatal": a
//!   self-referential array recurses until the depth guard trips
//!   (`_pr_cap_recursion_msg`) instead of looping forever.
//! - The caller persists the final `(_pr_cap_buf, _pr_cap_off)` byte range to a
//!   heap-owned string via `__rt_str_persist` — the returned PHP string is never
//!   an alias into the reused scratch buffer.
//! - Objects (tag 6) render as the bare `Array` header only, matching the
//!   existing stdout walker's documented limitation (full `ClassName Object`
//!   dumps need class metadata this runtime walker lacks).

use crate::codegen::abi;
use crate::codegen::{emit::Emitter, platform::Arch};

/// Maximum array/hash nesting depth before `print_r($v, true)` aborts with
/// `_pr_cap_recursion_msg` instead of recursing forever on a cyclic structure.
const MAX_DEPTH: i64 = 256;

/// `__rt_pr_cap_append`: append `len` bytes from `ptr` to `_pr_cap_buf` at the
/// current `_pr_cap_off` cursor, advancing the cursor. Fatal on overflow past
/// the fixed 1 MiB capture buffer. Input: AArch64 x1=ptr x2=len / x86_64 rsi=ptr rdx=len.
pub fn emit_pr_cap_append(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_pr_cap_append_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: pr_cap_append ---");
    emitter.label_global("__rt_pr_cap_append");

    emitter.instruction("cbz x2, __rt_pr_cap_append_done");                     // nothing to append
    abi::emit_symbol_address(emitter, "x9", "_pr_cap_off");                     // address of the write-cursor cell
    emitter.instruction("ldr x10, [x9]");                                       // reload the current cursor
    emitter.instruction("add x11, x10, x2");                                    // new cursor after this append
    emitter.instruction("mov x12, #1048576");                                   // the fixed capture-buffer capacity
    emitter.instruction("cmp x11, x12");                                        // does the append fit?
    emitter.instruction("b.hi __rt_pr_cap_overflow");                           // no: fatal, never silently truncate
    abi::emit_symbol_address(emitter, "x13", "_pr_cap_buf");                    // base of the capture buffer
    emitter.instruction("add x13, x13, x10");                                   // destination = base + current cursor
    emitter.instruction("mov x14, #0");                                         // byte-copy index
    emitter.label("__rt_pr_cap_append_loop");
    emitter.instruction("cmp x14, x2");                                         // copied every source byte?
    emitter.instruction("b.ge __rt_pr_cap_append_copy_done");
    emitter.instruction("ldrb w15, [x1, x14]");                                 // load the next source byte
    emitter.instruction("strb w15, [x13, x14]");                                // store it into the capture buffer
    emitter.instruction("add x14, x14, #1");                                    // advance the copy index
    emitter.instruction("b __rt_pr_cap_append_loop");
    emitter.label("__rt_pr_cap_append_copy_done");
    emitter.instruction("str x11, [x9]");                                       // publish the advanced cursor
    emitter.label("__rt_pr_cap_append_done");
    emitter.instruction("ret");

    emitter.label("__rt_pr_cap_overflow");
    emitter.instruction("mov x0, #2");                                          // fd = stderr for the overflow diagnostic
    abi::emit_symbol_address(emitter, "x1", "_pr_cap_overflow_msg");
    emitter.instruction(&format!(
        "mov x2, #{}",
        crate::codegen::runtime::data::PRINT_R_CAPTURE_OVERFLOW_MSG.len()
    ));
    emitter.syscall(4);
    emitter.instruction("mov x0, #1");                                          // exit code 1 for the fatal overflow abort
    emitter.syscall(1);
}

/// Emits the Linux x86_64 `__rt_pr_cap_append` variant.
fn emit_pr_cap_append_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: pr_cap_append ---");
    emitter.label_global("__rt_pr_cap_append");

    emitter.instruction("test rdx, rdx");                                       // nothing to append
    emitter.instruction("jz __rt_pr_cap_append_done_x86_64");
    abi::emit_symbol_address(emitter, "r9", "_pr_cap_off");                     // address of the write-cursor cell
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // reload the current cursor
    emitter.instruction("lea r11, [r10 + rdx]");                                // new cursor after this append
    emitter.instruction("cmp r11, 1048576");                                    // does the append fit?
    emitter.instruction("ja __rt_pr_cap_overflow_x86_64");                      // no: fatal, never silently truncate
    abi::emit_symbol_address(emitter, "r8", "_pr_cap_buf");                     // base of the capture buffer
    emitter.instruction("add r8, r10");                                         // destination = base + current cursor
    emitter.instruction("xor rcx, rcx");                                        // byte-copy index
    emitter.label("__rt_pr_cap_append_loop_x86_64");
    emitter.instruction("cmp rcx, rdx");                                        // copied every source byte?
    emitter.instruction("jge __rt_pr_cap_append_copy_done_x86_64");
    emitter.instruction("movzx eax, BYTE PTR [rsi + rcx]");                     // load the next source byte
    emitter.instruction("mov BYTE PTR [r8 + rcx], al");                         // store it into the capture buffer
    emitter.instruction("inc rcx");                                             // advance the copy index
    emitter.instruction("jmp __rt_pr_cap_append_loop_x86_64");
    emitter.label("__rt_pr_cap_append_copy_done_x86_64");
    emitter.instruction("mov QWORD PTR [r9], r11");                             // publish the advanced cursor
    emitter.label("__rt_pr_cap_append_done_x86_64");
    emitter.instruction("ret");

    emitter.label("__rt_pr_cap_overflow_x86_64");
    abi::emit_symbol_address(emitter, "rsi", "_pr_cap_overflow_msg");
    emitter.instruction(&format!(
        "mov edx, {}",
        crate::codegen::runtime::data::PRINT_R_CAPTURE_OVERFLOW_MSG.len()
    ));
    emitter.instruction("mov edi, 2");                                          // fd = stderr for the overflow diagnostic
    emitter.instruction("mov eax, 1");
    emitter.instruction("syscall");
    emitter.instruction("mov edi, 1");                                          // exit code 1 for the fatal overflow abort
    emitter.instruction("mov eax, 60");
    emitter.instruction("syscall");
}

/// `__rt_pr_cap_spaces`: append `n` ASCII spaces. Input: AArch64 x0 / x86_64 rdi = count.
pub fn emit_pr_cap_spaces(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_pr_cap_spaces_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: pr_cap_spaces ---");
    emitter.label_global("__rt_pr_cap_spaces");

    emitter.instruction("sub sp, sp, #32");                                     // allocate the helper frame
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish the helper frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // remaining space count

    emitter.label("__rt_pr_cap_spaces_loop");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the remaining count
    emitter.instruction("cmp x0, #0");                                          // any spaces left?
    emitter.instruction("b.le __rt_pr_cap_spaces_done");
    emitter.instruction("mov x9, #64");                                         // the pad buffer is 64 bytes wide
    emitter.instruction("cmp x0, x9");                                          // remaining vs the chunk cap
    emitter.instruction("csel x2, x0, x9, lt");                                 // chunk len = min(remaining, 64)
    emitter.instruction("sub x0, x0, x2");                                      // remaining -= chunk
    emitter.instruction("str x0, [sp, #0]");                                    // save the decremented count
    abi::emit_symbol_address(emitter, "x1", "_pr_spaces");                      // buffer = the shared 64-space pad
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append the space chunk
    emitter.instruction("b __rt_pr_cap_spaces_loop");

    emitter.label("__rt_pr_cap_spaces_done");
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the helper frame
    emitter.instruction("ret");
}

/// Emits the Linux x86_64 `__rt_pr_cap_spaces` variant.
fn emit_pr_cap_spaces_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: pr_cap_spaces ---");
    emitter.label_global("__rt_pr_cap_spaces");

    emitter.instruction("push rbp");                                            // save caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame pointer
    emitter.instruction("sub rsp, 16");                                         // allocate the helper frame
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // remaining space count

    emitter.label("__rt_pr_cap_spaces_loop_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the remaining count
    emitter.instruction("cmp rax, 0");                                          // any spaces left?
    emitter.instruction("jle __rt_pr_cap_spaces_done_x86_64");
    emitter.instruction("mov rdx, 64");                                         // the pad buffer is 64 bytes wide
    emitter.instruction("cmp rax, 64");                                         // remaining vs the chunk cap
    emitter.instruction("cmovl rdx, rax");                                      // chunk len = min(remaining, 64)
    emitter.instruction("sub rax, rdx");                                        // remaining -= chunk
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the decremented count
    abi::emit_symbol_address(emitter, "rsi", "_pr_spaces");                     // buffer = the shared 64-space pad
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append the space chunk
    emitter.instruction("jmp __rt_pr_cap_spaces_loop_x86_64");

    emitter.label("__rt_pr_cap_spaces_done_x86_64");
    emitter.instruction("add rsp, 16");                                         // release the helper frame
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");
}

/// `__rt_pr_cap_open`: append `<base spaces>(\n`. Input: AArch64 x0 / x86_64 rdi = base indent.
pub fn emit_pr_cap_open(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_pr_cap_paren_x86_64(emitter, "__rt_pr_cap_open", "_pr_open");
        return;
    }
    emit_pr_cap_paren_aarch64(emitter, "__rt_pr_cap_open", "_pr_open");
}

/// `__rt_pr_cap_close`: append `<base spaces>)\n`. Input: AArch64 x0 / x86_64 rdi = base indent.
pub fn emit_pr_cap_close(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_pr_cap_paren_x86_64(emitter, "__rt_pr_cap_close", "_pr_close");
        return;
    }
    emit_pr_cap_paren_aarch64(emitter, "__rt_pr_cap_close", "_pr_close");
}

/// Emits an AArch64 paren-line capture helper: indent `base` spaces then append the 2-byte `paren_sym`.
fn emit_pr_cap_paren_aarch64(emitter: &mut Emitter, label: &str, paren_sym: &str) {
    emitter.blank();
    emitter.comment(&format!("--- runtime: {} ---", &label[5..]));
    emitter.label_global(label);

    emitter.instruction("sub sp, sp, #16");                                     // allocate the helper frame
    emitter.instruction("stp x29, x30, [sp, #0]");                              // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish the helper frame pointer
    emitter.instruction("bl __rt_pr_cap_spaces");                               // x0 holds the base indent → pad it
    abi::emit_symbol_address(emitter, "x1", paren_sym);                         // load the `(\n` or `)\n` literal
    emitter.instruction("mov x2, #2");                                          // both paren literals are 2 bytes
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append the paren line
    emitter.instruction("ldp x29, x30, [sp, #0]");                              // restore frame pointer and return address
    emitter.instruction("add sp, sp, #16");                                     // release the helper frame
    emitter.instruction("ret");
}

/// Emits a Linux x86_64 paren-line capture helper: indent `base` spaces then append the 2-byte `paren_sym`.
fn emit_pr_cap_paren_x86_64(emitter: &mut Emitter, label: &str, paren_sym: &str) {
    emitter.blank();
    emitter.comment(&format!("--- runtime: {} ---", &label[5..]));
    emitter.label_global(label);

    emitter.instruction("push rbp");                                            // save caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame pointer
    emitter.instruction("call __rt_pr_cap_spaces");                             // rdi holds the base indent → pad it
    abi::emit_symbol_address(emitter, "rsi", paren_sym);                        // load the `(\n` or `)\n` literal
    emitter.instruction("mov edx, 2");                                          // both paren literals are 2 bytes
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append the paren line
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");
}

/// `__rt_pr_cap_int_key`: append `<indent spaces>[IDX] => ` for an integer key.
/// Input: AArch64 x0=idx x1=indent / x86_64 rdi=idx rsi=indent.
pub fn emit_pr_cap_int_key(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_pr_cap_int_key_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: pr_cap_int_key ---");
    emitter.label_global("__rt_pr_cap_int_key");

    emitter.instruction("sub sp, sp, #32");                                     // allocate the helper frame
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish the helper frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the integer key

    emitter.instruction("mov x0, x1");                                          // indent → spaces helper argument
    emitter.instruction("bl __rt_pr_cap_spaces");                               // pad the entry indent
    abi::emit_symbol_address(emitter, "x1", "_pr_lbrack");                      // load the `[` delimiter
    emitter.instruction("mov x2, #1");                                          // len("[") = 1
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append `[`
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the integer key
    emitter.instruction("bl __rt_itoa");                                        // x1=digits ptr, x2=digits len
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append the key digits
    abi::emit_symbol_address(emitter, "x1", "_pr_arrow");                       // load the `] => ` separator
    emitter.instruction("mov x2, #5");                                          // len("] => ") = 5
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append `] => `
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the helper frame
    emitter.instruction("ret");
}

/// Emits the Linux x86_64 `__rt_pr_cap_int_key` variant.
fn emit_pr_cap_int_key_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: pr_cap_int_key ---");
    emitter.label_global("__rt_pr_cap_int_key");

    emitter.instruction("push rbp");                                            // save caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame pointer
    emitter.instruction("sub rsp, 16");                                         // allocate the helper frame
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the integer key

    emitter.instruction("mov rdi, rsi");                                        // indent → spaces helper argument
    emitter.instruction("call __rt_pr_cap_spaces");                             // pad the entry indent
    abi::emit_symbol_address(emitter, "rsi", "_pr_lbrack");                     // load the `[` delimiter
    emitter.instruction("mov edx, 1");                                          // len("[") = 1
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append `[`
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the integer key
    emitter.instruction("call __rt_itoa");                                      // rax=digits ptr, rdx=digits len
    emitter.instruction("mov rsi, rax");                                        // digits ptr → append buffer
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append the key digits
    abi::emit_symbol_address(emitter, "rsi", "_pr_arrow");                      // load the `] => ` separator
    emitter.instruction("mov edx, 5");                                          // len("] => ") = 5
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append `] => `
    emitter.instruction("add rsp, 16");                                         // release the helper frame
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");
}

/// `__rt_pr_cap_str_key`: append `<indent spaces>[KEY] => ` for an unquoted string key.
/// Input: AArch64 x0=ptr x1=len x2=indent / x86_64 rdi=ptr rsi=len rdx=indent.
pub fn emit_pr_cap_str_key(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_pr_cap_str_key_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: pr_cap_str_key ---");
    emitter.label_global("__rt_pr_cap_str_key");

    emitter.instruction("sub sp, sp, #32");                                     // allocate the helper frame
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish the helper frame pointer
    emitter.instruction("stp x0, x1, [sp, #0]");                                // save the key ptr/len

    emitter.instruction("mov x0, x2");                                          // indent → spaces helper argument
    emitter.instruction("bl __rt_pr_cap_spaces");                               // pad the entry indent
    abi::emit_symbol_address(emitter, "x1", "_pr_lbrack");                      // load the `[` delimiter
    emitter.instruction("mov x2, #1");                                          // len("[") = 1
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append `[`
    emitter.instruction("ldr x1, [sp, #0]");                                    // reload the key ptr
    emitter.instruction("ldr x2, [sp, #8]");                                    // reload the key len
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append the raw (unquoted) key bytes
    abi::emit_symbol_address(emitter, "x1", "_pr_arrow");                       // load the `] => ` separator
    emitter.instruction("mov x2, #5");                                          // len("] => ") = 5
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append `] => `
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the helper frame
    emitter.instruction("ret");
}

/// Emits the Linux x86_64 `__rt_pr_cap_str_key` variant.
fn emit_pr_cap_str_key_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: pr_cap_str_key ---");
    emitter.label_global("__rt_pr_cap_str_key");

    emitter.instruction("push rbp");                                            // save caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame pointer
    emitter.instruction("sub rsp, 16");                                         // allocate the helper frame
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the key ptr
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the key len

    emitter.instruction("mov rdi, rdx");                                        // indent → spaces helper argument
    emitter.instruction("call __rt_pr_cap_spaces");                             // pad the entry indent
    abi::emit_symbol_address(emitter, "rsi", "_pr_lbrack");                     // load the `[` delimiter
    emitter.instruction("mov edx, 1");                                          // len("[") = 1
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append `[`
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // reload the key ptr
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // reload the key len
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append the raw (unquoted) key bytes
    abi::emit_symbol_address(emitter, "rsi", "_pr_arrow");                      // load the `] => ` separator
    emitter.instruction("mov edx, 5");                                          // len("] => ") = 5
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // append `] => `
    emitter.instruction("add rsp, 16");                                         // release the helper frame
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");
}

/// `__rt_pr_cap_value`: render one PHP value (no type wrapper) into the capture
/// buffer. Tags 4/5 recurse into the array walkers (bumping/checking
/// `_pr_cap_depth`), tag 7 unboxes a Mixed cell and redispatches, scalars
/// render directly, null/object render nothing/`Array` header only.
/// Input: AArch64 x0=tag x1=lo x2=hi x3=nested_base / x86_64 rdi=tag rsi=lo rdx=hi rcx=nested_base.
pub fn emit_pr_cap_value(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_pr_cap_value_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: pr_cap_value ---");
    emitter.label_global("__rt_pr_cap_value");

    emitter.instruction("sub sp, sp, #48");                                     // allocate the value frame
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish the value frame pointer
    emitter.instruction("str x1, [sp, #0]");                                    // save the value low word
    emitter.instruction("str x2, [sp, #8]");                                    // save the value high word
    emitter.instruction("str x3, [sp, #16]");                                   // save the nested paren base indent

    emitter.instruction("cmp x0, #7");                                          // boxed Mixed cell?
    emitter.instruction("b.eq __rt_pr_cap_val_mixed");
    emitter.instruction("cmp x0, #0");                                          // tag 0 = int
    emitter.instruction("b.eq __rt_pr_cap_val_int");
    emitter.instruction("cmp x0, #1");                                          // tag 1 = string
    emitter.instruction("b.eq __rt_pr_cap_val_str");
    emitter.instruction("cmp x0, #2");                                          // tag 2 = float
    emitter.instruction("b.eq __rt_pr_cap_val_flt");
    emitter.instruction("cmp x0, #3");                                          // tag 3 = bool
    emitter.instruction("b.eq __rt_pr_cap_val_bool");
    emitter.instruction("cmp x0, #4");                                          // tag 4 = indexed array
    emitter.instruction("b.eq __rt_pr_cap_val_arr");
    emitter.instruction("cmp x0, #5");                                          // tag 5 = hash
    emitter.instruction("b.eq __rt_pr_cap_val_hash");
    emitter.instruction("b __rt_pr_cap_val_done");                              // tag 6 object / 8 null → render nothing

    emitter.label("__rt_pr_cap_val_int");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the integer payload
    emitter.instruction("bl __rt_itoa");                                        // x1=digits ptr, x2=digits len
    abi::emit_call_label(emitter, "__rt_pr_cap_append");
    emitter.instruction("b __rt_pr_cap_val_done");

    emitter.label("__rt_pr_cap_val_str");
    emitter.instruction("ldr x1, [sp, #0]");                                    // reload the string ptr
    emitter.instruction("ldr x2, [sp, #8]");                                    // reload the string len
    abi::emit_call_label(emitter, "__rt_pr_cap_append");
    emitter.instruction("b __rt_pr_cap_val_done");

    emitter.label("__rt_pr_cap_val_flt");
    emitter.instruction("ldr d0, [sp, #0]");                                    // reload the float bit pattern
    emitter.instruction("bl __rt_ftoa");                                        // x1=text ptr, x2=text len
    abi::emit_call_label(emitter, "__rt_pr_cap_append");
    emitter.instruction("b __rt_pr_cap_val_done");

    emitter.label("__rt_pr_cap_val_bool");
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload the bool payload
    emitter.instruction("cbz x9, __rt_pr_cap_val_done");                        // false → render the empty string
    abi::emit_symbol_address(emitter, "x1", "_pr_one");                         // true → load the `1` literal
    emitter.instruction("mov x2, #1");                                          // len("1") = 1
    abi::emit_call_label(emitter, "__rt_pr_cap_append");
    emitter.instruction("b __rt_pr_cap_val_done");

    emitter.label("__rt_pr_cap_val_arr");
    abi::emit_symbol_address(emitter, "x1", "_pr_array_hdr");                   // load the `Array\n` header
    emitter.instruction("mov x2, #6");                                          // len("Array\n") = 6
    abi::emit_call_label(emitter, "__rt_pr_cap_append");
    emitter.instruction("ldr x0, [sp, #0]");                                    // nested indexed-array pointer
    emitter.instruction("ldr x1, [sp, #16]");                                   // base = the nested paren indent
    emitter.instruction("bl __rt_pr_cap_indexed");
    emitter.instruction("b __rt_pr_cap_val_done");

    emitter.label("__rt_pr_cap_val_hash");
    abi::emit_symbol_address(emitter, "x1", "_pr_array_hdr");                   // load the `Array\n` header
    emitter.instruction("mov x2, #6");                                          // len("Array\n") = 6
    abi::emit_call_label(emitter, "__rt_pr_cap_append");
    emitter.instruction("ldr x0, [sp, #0]");                                    // nested hash pointer
    emitter.instruction("ldr x1, [sp, #16]");                                   // base = the nested paren indent
    emitter.instruction("bl __rt_pr_cap_hash");
    emitter.instruction("b __rt_pr_cap_val_done");

    emitter.label("__rt_pr_cap_val_mixed");
    emitter.instruction("ldr x0, [sp, #0]");                                    // boxed Mixed cell pointer
    emitter.instruction("bl __rt_mixed_unbox");                                 // x0=inner tag, x1=lo, x2=hi
    emitter.instruction("ldr x3, [sp, #16]");                                   // reload the nested paren base indent
    emitter.instruction("bl __rt_pr_cap_value");                                // redispatch the unboxed scalar/array

    emitter.label("__rt_pr_cap_val_done");
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the value frame
    emitter.instruction("ret");
}

/// Emits the Linux x86_64 `__rt_pr_cap_value` variant.
fn emit_pr_cap_value_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: pr_cap_value ---");
    emitter.label_global("__rt_pr_cap_value");

    emitter.instruction("push rbp");                                            // save caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the value frame pointer
    emitter.instruction("sub rsp, 48");                                         // allocate the value frame
    emitter.instruction("mov QWORD PTR [rbp - 8], rsi");                        // save the value low word
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the value high word
    emitter.instruction("mov QWORD PTR [rbp - 24], rcx");                       // save the nested paren base indent
    emitter.instruction("mov rax, rdi");                                        // tag → dispatch register

    emitter.instruction("cmp rax, 7");                                          // boxed Mixed cell?
    emitter.instruction("je __rt_pr_cap_val_mixed_x86_64");
    emitter.instruction("cmp rax, 0");                                          // tag 0 = int
    emitter.instruction("je __rt_pr_cap_val_int_x86_64");
    emitter.instruction("cmp rax, 1");                                          // tag 1 = string
    emitter.instruction("je __rt_pr_cap_val_str_x86_64");
    emitter.instruction("cmp rax, 2");                                          // tag 2 = float
    emitter.instruction("je __rt_pr_cap_val_flt_x86_64");
    emitter.instruction("cmp rax, 3");                                          // tag 3 = bool
    emitter.instruction("je __rt_pr_cap_val_bool_x86_64");
    emitter.instruction("cmp rax, 4");                                          // tag 4 = indexed array
    emitter.instruction("je __rt_pr_cap_val_arr_x86_64");
    emitter.instruction("cmp rax, 5");                                          // tag 5 = hash
    emitter.instruction("je __rt_pr_cap_val_hash_x86_64");
    emitter.instruction("jmp __rt_pr_cap_val_done_x86_64");                     // tag 6 object / 8 null → render nothing

    emitter.label("__rt_pr_cap_val_int_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the integer payload
    emitter.instruction("call __rt_itoa");                                      // rax=digits ptr, rdx=digits len
    emitter.instruction("mov rsi, rax");
    abi::emit_call_label(emitter, "__rt_pr_cap_append");
    emitter.instruction("jmp __rt_pr_cap_val_done_x86_64");

    emitter.label("__rt_pr_cap_val_str_x86_64");
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // reload the string ptr
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // reload the string len
    abi::emit_call_label(emitter, "__rt_pr_cap_append");
    emitter.instruction("jmp __rt_pr_cap_val_done_x86_64");

    emitter.label("__rt_pr_cap_val_flt_x86_64");
    emitter.instruction("movsd xmm0, QWORD PTR [rbp - 8]");                     // reload the float bit pattern
    emitter.instruction("call __rt_ftoa");                                      // rax=text ptr, rdx=text len
    emitter.instruction("mov rsi, rax");
    abi::emit_call_label(emitter, "__rt_pr_cap_append");
    emitter.instruction("jmp __rt_pr_cap_val_done_x86_64");

    emitter.label("__rt_pr_cap_val_bool_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the bool payload
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_pr_cap_val_done_x86_64");                      // false → render the empty string
    abi::emit_symbol_address(emitter, "rsi", "_pr_one");                        // true → load the `1` literal
    emitter.instruction("mov edx, 1");                                          // len("1") = 1
    abi::emit_call_label(emitter, "__rt_pr_cap_append");
    emitter.instruction("jmp __rt_pr_cap_val_done_x86_64");

    emitter.label("__rt_pr_cap_val_arr_x86_64");
    abi::emit_symbol_address(emitter, "rsi", "_pr_array_hdr");                  // load the `Array\n` header
    emitter.instruction("mov edx, 6");                                          // len("Array\n") = 6
    abi::emit_call_label(emitter, "__rt_pr_cap_append");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // nested indexed-array pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // base = the nested paren indent
    emitter.instruction("call __rt_pr_cap_indexed");
    emitter.instruction("jmp __rt_pr_cap_val_done_x86_64");

    emitter.label("__rt_pr_cap_val_hash_x86_64");
    abi::emit_symbol_address(emitter, "rsi", "_pr_array_hdr");                  // load the `Array\n` header
    emitter.instruction("mov edx, 6");                                          // len("Array\n") = 6
    abi::emit_call_label(emitter, "__rt_pr_cap_append");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // nested hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // base = the nested paren indent
    emitter.instruction("call __rt_pr_cap_hash");
    emitter.instruction("jmp __rt_pr_cap_val_done_x86_64");

    emitter.label("__rt_pr_cap_val_mixed_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // boxed Mixed cell pointer → RAX
    emitter.instruction("call __rt_mixed_unbox");                               // rax=inner tag, rdi=lo, rdx=hi
    emitter.instruction("mov rsi, rdi");                                        // unboxed lo → value low argument
    emitter.instruction("mov rdi, rax");                                        // unboxed tag → value tag argument
    emitter.instruction("mov rcx, QWORD PTR [rbp - 24]");                       // reload the nested paren base indent
    emitter.instruction("call __rt_pr_cap_value");                              // redispatch the unboxed scalar/array

    emitter.label("__rt_pr_cap_val_done_x86_64");
    emitter.instruction("add rsp, 48");                                         // release the value frame
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");
}

/// `__rt_pr_cap_indexed`: render an indexed array body `<base>(\n ... <base>)\n`
/// into the capture buffer, self-dispatching each element on the array
/// value_type stamp. Bumps/checks `_pr_cap_depth`. Input:
/// AArch64 x0=arr x1=base / x86_64 rdi=arr rsi=base.
pub fn emit_pr_cap_indexed(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_pr_cap_indexed_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: pr_cap_indexed ---");
    emitter.label_global("__rt_pr_cap_indexed");

    // Frame (64 bytes): [0]arr [8]base [16]entry_indent [24]count [32]index
    //   [40]stamp [48]x29 [56]x30.
    emitter.instruction("sub sp, sp, #64");                                     // allocate the indexed-walk frame
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // establish the walk frame pointer
    emit_depth_guard_enter_aarch64(emitter);                                    // bump/check the nesting-depth guard
    emitter.instruction("str x0, [sp, #0]");                                    // save the array pointer
    emitter.instruction("str x1, [sp, #8]");                                    // save the paren base indent
    emitter.instruction("add x9, x1, #4");                                      // entry indent = base + 4
    emitter.instruction("str x9, [sp, #16]");                                   // save the entry indent
    emitter.instruction("ldr x10, [x0]");                                       // load the element count from the header
    emitter.instruction("str x10, [sp, #24]");                                  // save the element count
    emitter.instruction("str xzr, [sp, #32]");                                  // index = 0
    emitter.instruction("ldr x11, [x0, #-8]");                                  // load the packed array kind word
    emitter.instruction("lsr x11, x11, #8");                                    // shift the value_type stamp into the low byte
    emitter.instruction("and x11, x11, #0x0f");                                 // isolate the value_type field
    emitter.instruction("str x11, [sp, #40]");                                  // save the element value_type stamp

    emitter.instruction("ldr x0, [sp, #8]");                                    // base → open helper argument
    emitter.instruction("bl __rt_pr_cap_open");                                 // append `<base>(\n`

    emitter.label("__rt_pr_cap_idx_loop");
    emitter.instruction("ldr x9, [sp, #32]");                                   // reload the current index
    emitter.instruction("ldr x10, [sp, #24]");                                  // reload the element count
    emitter.instruction("cmp x9, x10");                                         // processed every element?
    emitter.instruction("b.ge __rt_pr_cap_idx_done");

    emitter.instruction("ldr x0, [sp, #32]");                                   // index → integer key
    emitter.instruction("ldr x1, [sp, #16]");                                   // entry indent → integer key
    emitter.instruction("bl __rt_pr_cap_int_key");                              // append `<indent>[N] => `

    emitter.instruction("ldr x12, [sp, #40]");                                  // reload the element stamp
    emitter.instruction("ldr x13, [sp, #0]");                                   // reload the array pointer
    emitter.instruction("ldr x14, [sp, #32]");                                  // reload the current index
    emitter.instruction("cmp x12, #1");                                         // string elements use a 16-byte stride
    emitter.instruction("b.eq __rt_pr_cap_idx_str");
    emitter.instruction("cmp x12, #7");                                         // mixed elements are boxed cells
    emitter.instruction("b.eq __rt_pr_cap_idx_mixed");

    // 8-byte-stride elements: int(0) / float(2) / bool(3) / array(4) / hash(5) / object(6).
    emitter.instruction("add x15, x14, #3");                                    // skip the 24-byte (3-quad) header
    emitter.instruction("ldr x1, [x13, x15, lsl #3]");                          // load the raw element word → value low
    emitter.instruction("mov x0, x12");                                         // tag = the array stamp
    emitter.instruction("mov x2, #0");                                          // high word unused for 8-byte elements
    emitter.instruction("ldr x3, [sp, #16]");                                   // entry indent
    emitter.instruction("add x3, x3, #4");                                      // nested base = entry indent + 4
    emitter.instruction("bl __rt_pr_cap_value");                                // render the element
    emitter.instruction("b __rt_pr_cap_idx_after");

    emitter.label("__rt_pr_cap_idx_str");
    emitter.instruction("lsl x15, x14, #4");                                    // index * 16
    emitter.instruction("add x15, x15, #24");                                   // element base offset = 24 + index*16
    emitter.instruction("add x15, x13, x15");                                   // element address
    emitter.instruction("ldr x1, [x15]");                                       // string ptr → value low
    emitter.instruction("ldr x2, [x15, #8]");                                   // string len → value high
    emitter.instruction("mov x0, #1");                                          // tag = string
    emitter.instruction("ldr x3, [sp, #16]");                                   // entry indent
    emitter.instruction("add x3, x3, #4");                                      // nested base = entry indent + 4
    emitter.instruction("bl __rt_pr_cap_value");                                // render the element
    emitter.instruction("b __rt_pr_cap_idx_after");

    emitter.label("__rt_pr_cap_idx_mixed");
    emitter.instruction("add x15, x14, #3");                                    // skip the 24-byte (3-quad) header
    emitter.instruction("ldr x15, [x13, x15, lsl #3]");                         // load the Mixed cell pointer
    emitter.instruction("ldr x0, [x15]");                                       // cell tag → value tag
    emitter.instruction("ldr x1, [x15, #8]");                                   // cell low word → value low
    emitter.instruction("ldr x2, [x15, #16]");                                  // cell high word → value high
    emitter.instruction("ldr x3, [sp, #16]");                                   // entry indent
    emitter.instruction("add x3, x3, #4");                                      // nested base = entry indent + 4
    emitter.instruction("bl __rt_pr_cap_value");                                // render the element

    emitter.label("__rt_pr_cap_idx_after");
    abi::emit_symbol_address(emitter, "x1", "_pr_nl");                         // load the line terminator
    emitter.instruction("mov x2, #1");                                          // len("\n") = 1
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // terminate the entry line
    emitter.instruction("ldr x9, [sp, #32]");                                   // reload the index
    emitter.instruction("add x9, x9, #1");                                      // advance the index
    emitter.instruction("str x9, [sp, #32]");                                   // save the updated index
    emitter.instruction("b __rt_pr_cap_idx_loop");

    emitter.label("__rt_pr_cap_idx_done");
    emitter.instruction("ldr x0, [sp, #8]");                                    // base → close helper argument
    emitter.instruction("bl __rt_pr_cap_close");                                // append `<base>)\n`
    emit_depth_guard_exit_aarch64(emitter);                                     // release the nesting-depth guard
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // release the indexed-walk frame
    emitter.instruction("ret");
}

/// Emits the Linux x86_64 `__rt_pr_cap_indexed` variant.
fn emit_pr_cap_indexed_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: pr_cap_indexed ---");
    emitter.label_global("__rt_pr_cap_indexed");

    // rbp-relative frame: [-8]arr [-16]base [-24]entry_indent [-32]count
    //   [-40]index [-48]stamp.
    emitter.instruction("push rbp");                                            // save caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the walk frame pointer
    emitter.instruction("sub rsp, 64");                                         // allocate the indexed-walk frame
    emit_depth_guard_enter_x86_64(emitter);                                     // bump/check the nesting-depth guard
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the array pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the paren base indent
    emitter.instruction("mov rax, rsi");                                        // copy the base indent
    emitter.instruction("add rax, 4");                                          // entry indent = base + 4
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the entry indent
    emitter.instruction("mov rax, QWORD PTR [rdi]");                            // load the element count from the header
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the element count
    emitter.instruction("mov QWORD PTR [rbp - 40], 0");                         // index = 0
    emitter.instruction("mov rax, QWORD PTR [rdi - 8]");                        // load the packed array kind word
    emitter.instruction("shr rax, 8");                                          // shift the value_type stamp into the low byte
    emitter.instruction("and rax, 0x0f");                                       // isolate the value_type field
    emitter.instruction("mov QWORD PTR [rbp - 48], rax");                       // save the element value_type stamp

    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // base → open helper argument
    emitter.instruction("call __rt_pr_cap_open");                               // append `<base>(\n`

    emitter.label("__rt_pr_cap_idx_loop_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // reload the current index
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");                       // reload the element count
    emitter.instruction("cmp rax, rcx");                                        // processed every element?
    emitter.instruction("jge __rt_pr_cap_idx_done_x86_64");

    emitter.instruction("mov rdi, QWORD PTR [rbp - 40]");                       // index → integer key
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // entry indent → integer key
    emitter.instruction("call __rt_pr_cap_int_key");                            // append `<indent>[N] => `

    emitter.instruction("mov r10, QWORD PTR [rbp - 48]");                       // reload the element stamp
    emitter.instruction("mov r9, QWORD PTR [rbp - 8]");                         // reload the array pointer
    emitter.instruction("mov r11, QWORD PTR [rbp - 40]");                       // reload the current index
    emitter.instruction("cmp r10, 1");                                          // string elements use a 16-byte stride
    emitter.instruction("je __rt_pr_cap_idx_str_x86_64");
    emitter.instruction("cmp r10, 7");                                          // mixed elements are boxed cells
    emitter.instruction("je __rt_pr_cap_idx_mixed_x86_64");

    // 8-byte-stride elements: int(0) / float(2) / bool(3) / array(4) / hash(5) / object(6).
    emitter.instruction("mov rax, r11");                                        // copy the index
    emitter.instruction("add rax, 3");                                          // skip the 24-byte (3-quad) header
    emitter.instruction("mov rsi, QWORD PTR [r9 + rax * 8]");                   // load the raw element word → value low
    emitter.instruction("mov rdi, r10");                                        // tag = the array stamp
    emitter.instruction("mov rdx, 0");                                          // high word unused for 8-byte elements
    emitter.instruction("mov rcx, QWORD PTR [rbp - 24]");                       // entry indent
    emitter.instruction("add rcx, 4");                                          // nested base = entry indent + 4
    emitter.instruction("call __rt_pr_cap_value");                              // render the element
    emitter.instruction("jmp __rt_pr_cap_idx_after_x86_64");

    emitter.label("__rt_pr_cap_idx_str_x86_64");
    emitter.instruction("mov rax, r11");                                        // copy the index
    emitter.instruction("shl rax, 4");                                          // index * 16
    emitter.instruction("add rax, 24");                                         // element base offset = 24 + index*16
    emitter.instruction("add rax, r9");                                         // element address
    emitter.instruction("mov rsi, QWORD PTR [rax]");                            // string ptr → value low
    emitter.instruction("mov rdx, QWORD PTR [rax + 8]");                        // string len → value high
    emitter.instruction("mov rdi, 1");                                          // tag = string
    emitter.instruction("mov rcx, QWORD PTR [rbp - 24]");                       // entry indent
    emitter.instruction("add rcx, 4");                                          // nested base = entry indent + 4
    emitter.instruction("call __rt_pr_cap_value");                              // render the element
    emitter.instruction("jmp __rt_pr_cap_idx_after_x86_64");

    emitter.label("__rt_pr_cap_idx_mixed_x86_64");
    emitter.instruction("mov rax, r11");                                        // copy the index
    emitter.instruction("add rax, 3");                                          // skip the 24-byte (3-quad) header
    emitter.instruction("mov rax, QWORD PTR [r9 + rax * 8]");                   // load the Mixed cell pointer
    emitter.instruction("mov rdi, QWORD PTR [rax]");                            // cell tag → value tag
    emitter.instruction("mov rsi, QWORD PTR [rax + 8]");                        // cell low word → value low
    emitter.instruction("mov rdx, QWORD PTR [rax + 16]");                       // cell high word → value high
    emitter.instruction("mov rcx, QWORD PTR [rbp - 24]");                       // entry indent
    emitter.instruction("add rcx, 4");                                          // nested base = entry indent + 4
    emitter.instruction("call __rt_pr_cap_value");                              // render the element

    emitter.label("__rt_pr_cap_idx_after_x86_64");
    abi::emit_symbol_address(emitter, "rsi", "_pr_nl");                        // load the line terminator
    emitter.instruction("mov edx, 1");                                          // len("\n") = 1
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // terminate the entry line
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // reload the index
    emitter.instruction("add rax, 1");                                          // advance the index
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // save the updated index
    emitter.instruction("jmp __rt_pr_cap_idx_loop_x86_64");

    emitter.label("__rt_pr_cap_idx_done_x86_64");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // base → close helper argument
    emitter.instruction("call __rt_pr_cap_close");                              // append `<base>)\n`
    emit_depth_guard_exit_x86_64(emitter);                                      // release the nesting-depth guard
    emitter.instruction("add rsp, 64");                                         // release the indexed-walk frame
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");
}

/// `__rt_pr_cap_hash`: render an associative-array body `<base>(\n ... <base>)\n`
/// into the capture buffer, iterating entries. Bumps/checks `_pr_cap_depth`.
/// Input: AArch64 x0=hash x1=base / x86_64 rdi=hash rsi=base.
pub fn emit_pr_cap_hash(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_pr_cap_hash_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: pr_cap_hash ---");
    emitter.label_global("__rt_pr_cap_hash");

    // Frame (112 bytes): [0]hash [8]base [16]entry_indent [24]count [32]cursor
    //   [40]items [48]key_ptr [56]key_len [64]val_lo [72]val_hi [80]val_tag
    //   [96]x29 [104]x30.
    emitter.instruction("sub sp, sp, #112");                                    // allocate the hash-walk frame
    emitter.instruction("stp x29, x30, [sp, #96]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #96");                                    // establish the walk frame pointer
    emit_depth_guard_enter_aarch64(emitter);                                    // bump/check the nesting-depth guard
    emitter.instruction("str x0, [sp, #0]");                                    // save the hash pointer
    emitter.instruction("str x1, [sp, #8]");                                    // save the paren base indent
    emitter.instruction("add x9, x1, #4");                                      // entry indent = base + 4
    emitter.instruction("str x9, [sp, #16]");                                   // save the entry indent
    emitter.instruction("ldr x0, [sp, #0]");                                    // hash → count helper argument
    emitter.instruction("bl __rt_hash_count");                                  // x0 = number of entries
    emitter.instruction("str x0, [sp, #24]");                                   // save the entry count
    emitter.instruction("str xzr, [sp, #32]");                                  // iterator cursor = 0
    emitter.instruction("str xzr, [sp, #40]");                                  // items emitted = 0

    emitter.instruction("ldr x0, [sp, #8]");                                    // base → open helper argument
    emitter.instruction("bl __rt_pr_cap_open");                                 // append `<base>(\n`

    emitter.label("__rt_pr_cap_hash_loop");
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload items emitted
    emitter.instruction("ldr x10, [sp, #24]");                                  // reload the entry count
    emitter.instruction("cmp x9, x10");                                         // processed every entry?
    emitter.instruction("b.ge __rt_pr_cap_hash_done");

    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the hash pointer
    emitter.instruction("ldr x1, [sp, #32]");                                   // reload the iterator cursor
    emitter.instruction("bl __rt_hash_iter_next");                              // x0=cursor, x1=key ptr, x2=key len, x3=val_lo, x4=val_hi, x5=val_tag
    emitter.instruction("str x0, [sp, #32]");                                   // save the next iterator cursor
    emitter.instruction("str x1, [sp, #48]");                                   // save the key ptr (or integer payload)
    emitter.instruction("str x2, [sp, #56]");                                   // save the key len (-1 for integer keys)
    emitter.instruction("str x3, [sp, #64]");                                   // save the value low word
    emitter.instruction("str x4, [sp, #72]");                                   // save the value high word
    emitter.instruction("str x5, [sp, #80]");                                   // save the value runtime tag

    emitter.instruction("ldr x2, [sp, #56]");                                   // reload the key len
    emitter.instruction("cmn x2, #1");                                          // integer key? (len == -1)
    emitter.instruction("b.eq __rt_pr_cap_hash_int_key");
    emitter.instruction("ldr x0, [sp, #48]");                                   // reload the key ptr
    emitter.instruction("ldr x1, [sp, #56]");                                   // reload the key len
    emitter.instruction("ldr x2, [sp, #16]");                                   // entry indent
    emitter.instruction("bl __rt_pr_cap_str_key");                              // append `<indent>[KEY] => `
    emitter.instruction("b __rt_pr_cap_hash_after_key");
    emitter.label("__rt_pr_cap_hash_int_key");
    emitter.instruction("ldr x0, [sp, #48]");                                   // integer key payload → integer key
    emitter.instruction("ldr x1, [sp, #16]");                                   // entry indent → integer key
    emitter.instruction("bl __rt_pr_cap_int_key");                              // append `<indent>[N] => `

    emitter.label("__rt_pr_cap_hash_after_key");
    emitter.instruction("ldr x0, [sp, #80]");                                   // value tag → value renderer
    emitter.instruction("ldr x1, [sp, #64]");                                   // value low → value renderer
    emitter.instruction("ldr x2, [sp, #72]");                                   // value high → value renderer
    emitter.instruction("ldr x3, [sp, #16]");                                   // entry indent
    emitter.instruction("add x3, x3, #4");                                      // nested base = entry indent + 4
    emitter.instruction("bl __rt_pr_cap_value");                                // render the entry value

    abi::emit_symbol_address(emitter, "x1", "_pr_nl");                         // load the line terminator
    emitter.instruction("mov x2, #1");                                          // len("\n") = 1
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // terminate the entry line
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload items emitted
    emitter.instruction("add x9, x9, #1");                                      // count this entry
    emitter.instruction("str x9, [sp, #40]");                                   // save the updated item count
    emitter.instruction("b __rt_pr_cap_hash_loop");

    emitter.label("__rt_pr_cap_hash_done");
    emitter.instruction("ldr x0, [sp, #8]");                                    // base → close helper argument
    emitter.instruction("bl __rt_pr_cap_close");                                // append `<base>)\n`
    emit_depth_guard_exit_aarch64(emitter);                                     // release the nesting-depth guard
    emitter.instruction("ldp x29, x30, [sp, #96]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #112");                                    // release the hash-walk frame
    emitter.instruction("ret");
}

/// Emits the Linux x86_64 `__rt_pr_cap_hash` variant.
fn emit_pr_cap_hash_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: pr_cap_hash ---");
    emitter.label_global("__rt_pr_cap_hash");

    // rbp-relative frame: [-8]hash [-16]base [-24]entry_indent [-32]count
    //   [-40]cursor [-48]items [-56]key_ptr [-64]key_len [-72]val_lo
    //   [-80]val_hi [-88]val_tag.
    emitter.instruction("push rbp");                                            // save caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the walk frame pointer
    emitter.instruction("sub rsp, 112");                                        // allocate the hash-walk frame
    emit_depth_guard_enter_x86_64(emitter);                                     // bump/check the nesting-depth guard
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the hash pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the paren base indent
    emitter.instruction("mov rax, rsi");                                        // copy the base indent
    emitter.instruction("add rax, 4");                                          // entry indent = base + 4
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the entry indent
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // hash → count helper argument
    emitter.instruction("call __rt_hash_count");                                // rax = number of entries
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the entry count
    emitter.instruction("mov QWORD PTR [rbp - 40], 0");                         // iterator cursor = 0
    emitter.instruction("mov QWORD PTR [rbp - 48], 0");                         // items emitted = 0

    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // base → open helper argument
    emitter.instruction("call __rt_pr_cap_open");                               // append `<base>(\n`

    emitter.label("__rt_pr_cap_hash_loop_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 48]");                       // reload items emitted
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");                       // reload the entry count
    emitter.instruction("cmp rax, rcx");                                        // processed every entry?
    emitter.instruction("jge __rt_pr_cap_hash_done_x86_64");

    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 40]");                       // reload the iterator cursor
    emitter.instruction("call __rt_hash_iter_next");                            // rax=cursor, rdi=key ptr, rdx=key len, rcx=val_lo, r8=val_hi, r9=val_tag
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // save the next iterator cursor
    emitter.instruction("mov QWORD PTR [rbp - 56], rdi");                       // save the key ptr (or integer payload)
    emitter.instruction("mov QWORD PTR [rbp - 64], rdx");                       // save the key len (-1 for integer keys)
    emitter.instruction("mov QWORD PTR [rbp - 72], rcx");                       // save the value low word
    emitter.instruction("mov QWORD PTR [rbp - 80], r8");                        // save the value high word
    emitter.instruction("mov QWORD PTR [rbp - 88], r9");                        // save the value runtime tag

    emitter.instruction("mov rdx, QWORD PTR [rbp - 64]");                       // reload the key len
    emitter.instruction("cmp rdx, -1");                                         // integer key?
    emitter.instruction("je __rt_pr_cap_hash_int_key_x86_64");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 56]");                       // reload the key ptr
    emitter.instruction("mov rsi, QWORD PTR [rbp - 64]");                       // reload the key len
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // entry indent
    emitter.instruction("call __rt_pr_cap_str_key");                            // append `<indent>[KEY] => `
    emitter.instruction("jmp __rt_pr_cap_hash_after_key_x86_64");
    emitter.label("__rt_pr_cap_hash_int_key_x86_64");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 56]");                       // integer key payload → integer key
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // entry indent → integer key
    emitter.instruction("call __rt_pr_cap_int_key");                            // append `<indent>[N] => `

    emitter.label("__rt_pr_cap_hash_after_key_x86_64");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 88]");                       // value tag → value renderer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 72]");                       // value low → value renderer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 80]");                       // value high → value renderer
    emitter.instruction("mov rcx, QWORD PTR [rbp - 24]");                       // entry indent
    emitter.instruction("add rcx, 4");                                          // nested base = entry indent + 4
    emitter.instruction("call __rt_pr_cap_value");                              // render the entry value

    abi::emit_symbol_address(emitter, "rsi", "_pr_nl");                        // load the line terminator
    emitter.instruction("mov edx, 1");                                          // len("\n") = 1
    abi::emit_call_label(emitter, "__rt_pr_cap_append");                        // terminate the entry line
    emitter.instruction("mov rax, QWORD PTR [rbp - 48]");                       // reload items emitted
    emitter.instruction("add rax, 1");                                          // count this entry
    emitter.instruction("mov QWORD PTR [rbp - 48], rax");                       // save the updated item count
    emitter.instruction("jmp __rt_pr_cap_hash_loop_x86_64");

    emitter.label("__rt_pr_cap_hash_done_x86_64");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // base → close helper argument
    emitter.instruction("call __rt_pr_cap_close");                              // append `<base>)\n`
    emit_depth_guard_exit_x86_64(emitter);                                      // release the nesting-depth guard
    emitter.instruction("add rsp, 112");                                        // release the hash-walk frame
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");
}

/// Bumps `_pr_cap_depth` and fatals past `MAX_DEPTH` (AArch64). Uses x9/x10 scratch.
fn emit_depth_guard_enter_aarch64(emitter: &mut Emitter) {
    abi::emit_symbol_address(emitter, "x9", "_pr_cap_depth");
    emitter.instruction("ldr x10, [x9]");                                       // reload the current nesting depth
    emitter.instruction("add x10, x10, #1");                                    // enter one more nesting level
    emitter.instruction(&format!("cmp x10, #{}", MAX_DEPTH));                   // exceeded the depth guard?
    emitter.instruction("b.gt __rt_pr_cap_recursion_fatal");                    // never loop forever on a cyclic array
    emitter.instruction("str x10, [x9]");                                       // publish the bumped depth
}

/// Decrements `_pr_cap_depth` on the way back out of one nesting level (AArch64).
fn emit_depth_guard_exit_aarch64(emitter: &mut Emitter) {
    abi::emit_symbol_address(emitter, "x9", "_pr_cap_depth");
    emitter.instruction("ldr x10, [x9]");                                       // reload the current nesting depth
    emitter.instruction("sub x10, x10, #1");                                    // leave this nesting level
    emitter.instruction("str x10, [x9]");                                       // publish the decremented depth
}

/// Bumps `_pr_cap_depth` and fatals past `MAX_DEPTH` (x86_64). Uses rax/r10 scratch.
fn emit_depth_guard_enter_x86_64(emitter: &mut Emitter) {
    abi::emit_symbol_address(emitter, "r10", "_pr_cap_depth");
    emitter.instruction("mov rax, QWORD PTR [r10]");                            // reload the current nesting depth
    emitter.instruction("inc rax");                                             // enter one more nesting level
    emitter.instruction(&format!("cmp rax, {}", MAX_DEPTH));                    // exceeded the depth guard?
    emitter.instruction("jg __rt_pr_cap_recursion_fatal_x86_64");               // never loop forever on a cyclic array
    emitter.instruction("mov QWORD PTR [r10], rax");                            // publish the bumped depth
}

/// Decrements `_pr_cap_depth` on the way back out of one nesting level (x86_64).
fn emit_depth_guard_exit_x86_64(emitter: &mut Emitter) {
    abi::emit_symbol_address(emitter, "r10", "_pr_cap_depth");
    emitter.instruction("mov rax, QWORD PTR [r10]");                            // reload the current nesting depth
    emitter.instruction("dec rax");                                             // leave this nesting level
    emitter.instruction("mov QWORD PTR [r10], rax");                            // publish the decremented depth
}

/// `__rt_pr_cap_recursion_fatal`: shared abort target for the depth guard (both
/// `emit_pr_cap_indexed`/`emit_pr_cap_hash` share this single fatal label).
pub fn emit_pr_cap_recursion_fatal(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emitter.blank();
        emitter.comment("--- runtime: pr_cap_recursion_fatal ---");
        emitter.label_global("__rt_pr_cap_recursion_fatal_x86_64");
        abi::emit_symbol_address(emitter, "rsi", "_pr_cap_recursion_msg");
        emitter.instruction(&format!(
            "mov edx, {}",
            crate::codegen::runtime::data::PRINT_R_CAPTURE_RECURSION_MSG.len()
        ));
        emitter.instruction("mov edi, 2");                                      // fd = stderr for the recursion diagnostic
        emitter.instruction("mov eax, 1");
        emitter.instruction("syscall");
        emitter.instruction("mov edi, 1");                                      // exit code 1 for the fatal recursion abort
        emitter.instruction("mov eax, 60");
        emitter.instruction("syscall");
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: pr_cap_recursion_fatal ---");
    emitter.label_global("__rt_pr_cap_recursion_fatal");
    emitter.instruction("mov x0, #2");                                          // fd = stderr for the recursion diagnostic
    abi::emit_symbol_address(emitter, "x1", "_pr_cap_recursion_msg");
    emitter.instruction(&format!(
        "mov x2, #{}",
        crate::codegen::runtime::data::PRINT_R_CAPTURE_RECURSION_MSG.len()
    ));
    emitter.syscall(4);
    emitter.instruction("mov x0, #1");                                          // exit code 1 for the fatal recursion abort
    emitter.syscall(1);
}
