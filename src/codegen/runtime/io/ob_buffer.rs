//! Purpose:
//! Emits the `__rt_ob_*` runtime helpers backing PHP output buffering
//! (`ob_start()`/`ob_get_clean()`/`ob_end_flush()`/`ob_end_clean()`/
//! `ob_get_contents()`/`ob_get_level()`) and the `__rt_headers_sent` flag read
//! by `headers_sent()`.
//!
//! Called from:
//! - `crate::codegen::runtime::emitters::emit_runtime()` via `crate::codegen::runtime::io`.
//! - `crate::codegen::runtime::io::stdout_write` (`__rt_stdout_write` routes
//!   through `__rt_ob_append` while a buffer is active, and stamps
//!   `_headers_sent` on the real-write path).
//! - `crate::codegen_ir::lower_inst::builtins::output_buffering` (the EIR
//!   lowering for the `ob_*`/`headers_sent`/`flush`/`header_remove` builtins).
//!
//! Key details:
//! - A fixed LIFO stack of `OB_MAX_LEVELS` (16) scratch buffers, each capped at
//!   `OB_LEVEL_CAP` (1 MiB) — the same fixed-capacity/loud-overflow shape as
//!   `print_r_capture.rs`'s `_pr_cap_buf`. `_ob_level` (declared in
//!   `crate::codegen::runtime::data::fixed`) is the current nesting depth (0 =
//!   no buffer active). Level N's bytes live at `_ob_bufs + (N-1)*OB_LEVEL_CAP`
//!   with write cursor `_ob_offs + (N-1)*8`.
//! - `__rt_ob_append` is called by `__rt_stdout_write` with the SAME (ptr, len)
//!   registers `__rt_stdout_write` itself receives, so no register shuffling is
//!   needed at that call site. It assumes `_ob_level >= 1` (the caller checks
//!   first) and is a loud runtime fatal on a full level, never silent truncation.
//! - `__rt_ob_peek_contents` returns an OWNED heap copy (via `__rt_str_persist`)
//!   of the current top level's bytes, or a null pointer sentinel when no
//!   buffer is active — the exact shape `box_owned_string_or_false_result`
//!   (shared with `realpath()`) expects, so `ob_get_contents()`/`ob_get_clean()`
//!   box it as `string|false` without any new boxing logic. It does NOT pop the
//!   level; `ob_get_clean()` calls `__rt_ob_pop` afterward.
//! - `__rt_ob_pop` is a defensive no-op at level 0 so callers never need to
//!   branch around it.
//! - `__rt_ob_end_flush` decrements `_ob_level` BEFORE re-invoking
//!   `__rt_stdout_write` with the popped level's bytes, so the flushed content
//!   correctly lands in the next-enclosing buffer (if any) or the real
//!   syscall/`--web` capture path — mirrors PHP's nested `ob_start()` +
//!   `ob_end_flush()` write-through (php -n verified).
//! - `__rt_ob_end_clean`/`__rt_ob_end_flush` write a PHP-style notice to stderr
//!   and return false on an empty stack (php -n verified: "Failed to
//!   delete[,] and flush buffer. No buffer to delete or flush").

use crate::codegen::{abi, emit::Emitter, platform::Arch};

/// Maximum simultaneous `ob_start()` nesting depth before a loud runtime fatal.
pub(crate) const OB_MAX_LEVELS: usize = 16;
/// Fixed per-level output-buffer capacity in bytes (1 MiB, matching `_pr_cap_buf`).
pub(crate) const OB_LEVEL_CAP: usize = 1_048_576;
/// `OB_LEVEL_CAP` expressed as a left-shift amount (1 MiB == 1 << 20), used to
/// multiply a level index by the per-level capacity with a single shift.
const OB_LEVEL_CAP_SHIFT: u32 = 20;

const _: () = assert!(1usize << OB_LEVEL_CAP_SHIFT == OB_LEVEL_CAP, "OB_LEVEL_CAP_SHIFT must match OB_LEVEL_CAP");

/// Emits a stderr write + `exit(1)` fatal-abort tail, sharing the `_ob_*_msg`
/// overflow diagnostics between `__rt_ob_start` and `__rt_ob_append`.
fn emit_ob_fatal_aarch64(emitter: &mut Emitter, msg_symbol: &str, msg_len: usize) {
    emitter.instruction("mov x0, #2");                                          // fd = stderr for the fatal diagnostic
    abi::emit_symbol_address(emitter, "x1", msg_symbol);
    emitter.instruction(&format!("mov x2, #{}", msg_len));                      // message byte length
    emitter.syscall(4);
    emitter.instruction("mov x0, #1");                                          // exit code 1 for the fatal abort
    emitter.syscall(1);
}

/// x86_64 (Linux) counterpart of `emit_ob_fatal_aarch64`.
fn emit_ob_fatal_x86_64(emitter: &mut Emitter, msg_symbol: &str, msg_len: usize) {
    abi::emit_symbol_address(emitter, "rsi", msg_symbol);
    emitter.instruction(&format!("mov edx, {}", msg_len));                      // message byte length
    emitter.instruction("mov edi, 2");                                          // fd = stderr for the fatal diagnostic
    emitter.instruction("mov eax, 1");                                          // Linux x86_64 syscall 1 = write
    emitter.instruction("syscall");
    emitter.instruction("mov edi, 1");                                          // exit code 1 for the fatal abort
    emitter.instruction("mov eax, 60");                                         // Linux x86_64 syscall 60 = exit
    emitter.instruction("syscall");
}

/// Emits `__rt_ob_start`: pushes a new output-buffering level. Fatal
/// (`_ob_max_levels_msg`) when the fixed `OB_MAX_LEVELS` nesting cap would be
/// exceeded. No arguments; always returns `1` (true) in the int result register
/// on success — elephc only supports the plain, callback-free `ob_start()` form
/// (`ob_start($callback)`/`ob_start(..., $chunk_size)` are rejected at the
/// signature level, so this helper never sees those arguments).
pub fn emit_ob_start(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ob_start_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ob_start ---");
    emitter.label_global("__rt_ob_start");

    abi::emit_symbol_address(emitter, "x9", "_ob_level");
    emitter.instruction("ldr x10, [x9]");                                       // current nesting depth
    emitter.instruction(&format!("cmp x10, #{}", OB_MAX_LEVELS));               // would this push exceed the fixed cap?
    emitter.instruction("b.ge __rt_ob_start_overflow");
    emitter.instruction("add x11, x10, #1");                                    // new nesting depth
    emitter.instruction("str x11, [x9]");                                       // publish the new depth
    abi::emit_symbol_address(emitter, "x12", "_ob_offs");
    emitter.instruction("lsl x13, x10, #3");                                    // (new_level - 1) * 8 == old_level * 8
    emitter.instruction("str xzr, [x12, x13]");                                 // zero the new level's write cursor
    emitter.instruction("mov x0, #1");                                          // ob_start() always returns true
    emitter.instruction("ret");

    emitter.label("__rt_ob_start_overflow");
    emit_ob_fatal_aarch64(emitter, "_ob_max_levels_msg", crate::codegen::runtime::data::OB_MAX_LEVELS_MSG.len());
}

/// x86_64 (Linux) counterpart of `emit_ob_start`.
fn emit_ob_start_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ob_start ---");
    emitter.label_global("__rt_ob_start");

    abi::emit_symbol_address(emitter, "r8", "_ob_level");
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // current nesting depth
    emitter.instruction(&format!("cmp r9, {}", OB_MAX_LEVELS));                 // would this push exceed the fixed cap?
    emitter.instruction("jge __rt_ob_start_overflow_x86_64");
    emitter.instruction("lea r10, [r9 + 1]");                                   // new nesting depth
    emitter.instruction("mov QWORD PTR [r8], r10");                             // publish the new depth
    abi::emit_symbol_address(emitter, "r11", "_ob_offs");
    emitter.instruction("shl r9, 3");                                           // (new_level - 1) * 8 == old_level * 8
    emitter.instruction("mov QWORD PTR [r11 + r9], 0");                         // zero the new level's write cursor
    emitter.instruction("mov eax, 1");                                          // ob_start() always returns true
    emitter.instruction("ret");

    emitter.label("__rt_ob_start_overflow_x86_64");
    emit_ob_fatal_x86_64(emitter, "_ob_max_levels_msg", crate::codegen::runtime::data::OB_MAX_LEVELS_MSG.len());
}

/// Emits `__rt_ob_append`: appends `len` bytes from `ptr` to the CURRENT top
/// output-buffering level. Called only by `__rt_stdout_write` while
/// `_ob_level >= 1` (the caller checks first), so this never touches level 0.
/// Input: AArch64 `x0`=ptr `x1`=len (the same registers `__rt_stdout_write`
/// itself receives) / x86_64 `rdi`=ptr `rsi`=len. Fatal (`_ob_overflow_msg`)
/// when the append would exceed the level's fixed `OB_LEVEL_CAP` (1 MiB).
pub fn emit_ob_append(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ob_append_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ob_append ---");
    emitter.label_global("__rt_ob_append");

    emitter.instruction("cbz x1, __rt_ob_append_done");                         // nothing to append
    abi::emit_symbol_address(emitter, "x9", "_ob_level");
    emitter.instruction("ldr x10, [x9]");                                       // current level
    emitter.instruction("sub x10, x10, #1");                                    // idx = level - 1
    abi::emit_symbol_address(emitter, "x9", "_ob_offs");
    emitter.instruction("lsl x11, x10, #3");                                    // idx * 8
    emitter.instruction("add x9, x9, x11");                                     // &_ob_offs[idx]
    emitter.instruction("ldr x12, [x9]");                                       // this level's write cursor
    emitter.instruction("add x13, x12, x1");                                    // cursor after this append
    emitter.instruction(&format!("mov x14, #{}", OB_LEVEL_CAP));                // this level's fixed capacity
    emitter.instruction("cmp x13, x14");                                        // does the append fit?
    emitter.instruction("b.hi __rt_ob_append_overflow");                        // no: fatal, never silently truncate
    emitter.instruction("str x13, [x9]");                                       // publish the advanced cursor
    abi::emit_symbol_address(emitter, "x14", "_ob_bufs");
    emitter.instruction(&format!("lsl x11, x10, #{}", OB_LEVEL_CAP_SHIFT));     // idx * OB_LEVEL_CAP
    emitter.instruction("add x14, x14, x11");                                   // this level's buffer base
    emitter.instruction("add x14, x14, x12");                                   // + old cursor = destination
    emitter.instruction("mov x15, #0");                                         // byte-copy index
    emitter.label("__rt_ob_append_loop");
    emitter.instruction("cmp x15, x1");                                         // copied every source byte?
    emitter.instruction("b.ge __rt_ob_append_done");
    emitter.instruction("ldrb w16, [x0, x15]");                                 // load the next source byte
    emitter.instruction("strb w16, [x14, x15]");                                // store it into the output buffer
    emitter.instruction("add x15, x15, #1");                                    // advance the copy index
    emitter.instruction("b __rt_ob_append_loop");

    emitter.label("__rt_ob_append_done");
    emitter.instruction("ret");

    emitter.label("__rt_ob_append_overflow");
    emit_ob_fatal_aarch64(emitter, "_ob_overflow_msg", crate::codegen::runtime::data::OB_OVERFLOW_MSG.len());
}

/// x86_64 (Linux) counterpart of `emit_ob_append`.
fn emit_ob_append_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ob_append ---");
    emitter.label_global("__rt_ob_append");

    emitter.instruction("test rsi, rsi");                                       // nothing to append
    emitter.instruction("jz __rt_ob_append_done_x86_64");
    abi::emit_symbol_address(emitter, "r8", "_ob_level");
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // current level
    emitter.instruction("dec r9");                                              // idx = level - 1
    abi::emit_symbol_address(emitter, "r8", "_ob_offs");
    emitter.instruction("lea r8, [r8 + r9*8]");                                 // &_ob_offs[idx]
    emitter.instruction("mov r10, QWORD PTR [r8]");                             // this level's write cursor
    emitter.instruction("lea r11, [r10 + rsi]");                                // cursor after this append
    emitter.instruction(&format!("cmp r11, {}", OB_LEVEL_CAP));                 // does the append fit?
    emitter.instruction("ja __rt_ob_append_overflow_x86_64");                   // no: fatal, never silently truncate
    emitter.instruction("mov QWORD PTR [r8], r11");                             // publish the advanced cursor
    abi::emit_symbol_address(emitter, "rax", "_ob_bufs");
    emitter.instruction(&format!("shl r9, {}", OB_LEVEL_CAP_SHIFT));            // idx * OB_LEVEL_CAP
    emitter.instruction("add rax, r9");                                         // this level's buffer base
    emitter.instruction("add rax, r10");                                        // + old cursor = destination
    emitter.instruction("xor rcx, rcx");                                        // byte-copy index
    emitter.label("__rt_ob_append_loop_x86_64");
    emitter.instruction("cmp rcx, rsi");                                        // copied every source byte?
    emitter.instruction("jge __rt_ob_append_done_x86_64");
    emitter.instruction("movzx r10d, BYTE PTR [rdi + rcx]");                    // load the next source byte
    emitter.instruction("mov BYTE PTR [rax + rcx], r10b");                      // store it into the output buffer
    emitter.instruction("inc rcx");                                             // advance the copy index
    emitter.instruction("jmp __rt_ob_append_loop_x86_64");

    emitter.label("__rt_ob_append_done_x86_64");
    emitter.instruction("ret");

    emitter.label("__rt_ob_append_overflow_x86_64");
    emit_ob_fatal_x86_64(emitter, "_ob_overflow_msg", crate::codegen::runtime::data::OB_OVERFLOW_MSG.len());
}

/// Emits `__rt_ob_peek_contents`: returns an OWNED heap copy of the current top
/// output-buffering level's bytes (persisted via `__rt_str_persist`), in the
/// same (ptr, len) shape `box_owned_string_or_false_result` expects (a null
/// pointer means "no buffer active" / `false`). Never pops a level.
pub fn emit_ob_peek_contents(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ob_peek_contents_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ob_peek_contents ---");
    emitter.label_global("__rt_ob_peek_contents");

    // -- minimal frame: the active branch calls __rt_str_persist, which clobbers x30 --
    emitter.instruction("stp x29, x30, [sp, #-16]!");
    emitter.instruction("mov x29, sp");

    abi::emit_symbol_address(emitter, "x9", "_ob_level");
    emitter.instruction("ldr x10, [x9]");                                       // current level
    emitter.instruction("cbnz x10, __rt_ob_peek_contents_active");
    emitter.instruction("mov x1, #0");                                          // no buffer active: null pointer sentinel
    emitter.instruction("mov x2, #0");
    emitter.instruction("ldp x29, x30, [sp], #16");                             // restore frame pointer and return address
    emitter.instruction("ret");

    emitter.label("__rt_ob_peek_contents_active");
    emitter.instruction("sub x10, x10, #1");                                    // idx = level - 1
    abi::emit_symbol_address(emitter, "x9", "_ob_offs");
    emitter.instruction("lsl x11, x10, #3");
    emitter.instruction("ldr x2, [x9, x11]");                                   // this level's byte count
    abi::emit_symbol_address(emitter, "x1", "_ob_bufs");
    emitter.instruction(&format!("lsl x11, x10, #{}", OB_LEVEL_CAP_SHIFT));
    emitter.instruction("add x1, x1, x11");                                     // this level's buffer base (start of its bytes)
    abi::emit_call_label(emitter, "__rt_str_persist");                          // copy the level's bytes to an owned heap string
    emitter.instruction("ldp x29, x30, [sp], #16");                             // restore frame pointer and return address
    emitter.instruction("ret");
}

/// x86_64 (Linux) counterpart of `emit_ob_peek_contents`.
fn emit_ob_peek_contents_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ob_peek_contents ---");
    emitter.label_global("__rt_ob_peek_contents");

    abi::emit_symbol_address(emitter, "r8", "_ob_level");
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // current level
    emitter.instruction("test r9, r9");
    emitter.instruction("jnz __rt_ob_peek_contents_active_x86_64");
    emitter.instruction("xor eax, eax");                                        // no buffer active: null pointer sentinel
    emitter.instruction("xor edx, edx");
    emitter.instruction("ret");

    emitter.label("__rt_ob_peek_contents_active_x86_64");
    emitter.instruction("dec r9");                                              // idx = level - 1
    abi::emit_symbol_address(emitter, "r8", "_ob_offs");
    emitter.instruction("mov rdx, QWORD PTR [r8 + r9*8]");                      // this level's byte count
    abi::emit_symbol_address(emitter, "rax", "_ob_bufs");
    emitter.instruction(&format!("shl r9, {}", OB_LEVEL_CAP_SHIFT));
    emitter.instruction("add rax, r9");                                         // this level's buffer base (start of its bytes)
    abi::emit_call_label(emitter, "__rt_str_persist");                          // copy the level's bytes to an owned heap string
    emitter.instruction("ret");
}

/// Emits `__rt_ob_pop`: decrements `_ob_level` by one, or does nothing at
/// level 0 (defensive no-op so callers never need to branch around it).
pub fn emit_ob_pop(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ob_pop_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ob_pop ---");
    emitter.label_global("__rt_ob_pop");

    abi::emit_symbol_address(emitter, "x9", "_ob_level");
    emitter.instruction("ldr x10, [x9]");
    emitter.instruction("cbz x10, __rt_ob_pop_done");                           // nothing active: no-op
    emitter.instruction("sub x10, x10, #1");
    emitter.instruction("str x10, [x9]");
    emitter.label("__rt_ob_pop_done");
    emitter.instruction("ret");
}

/// x86_64 (Linux) counterpart of `emit_ob_pop`.
fn emit_ob_pop_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ob_pop ---");
    emitter.label_global("__rt_ob_pop");

    abi::emit_symbol_address(emitter, "r8", "_ob_level");
    emitter.instruction("mov r9, QWORD PTR [r8]");
    emitter.instruction("test r9, r9");
    emitter.instruction("jz __rt_ob_pop_done_x86_64");                          // nothing active: no-op
    emitter.instruction("dec r9");
    emitter.instruction("mov QWORD PTR [r8], r9");
    emitter.label("__rt_ob_pop_done_x86_64");
    emitter.instruction("ret");
}

/// Emits `__rt_ob_end_clean`: discards the current top output-buffering level
/// without flushing it. Returns `1` (true) in the int result register on
/// success, `0` (false) plus a PHP-style stderr notice on an empty stack
/// (php -n verified: "Failed to delete buffer. No buffer to delete").
pub fn emit_ob_end_clean(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ob_end_clean_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ob_end_clean ---");
    emitter.label_global("__rt_ob_end_clean");

    abi::emit_symbol_address(emitter, "x9", "_ob_level");
    emitter.instruction("ldr x10, [x9]");
    emitter.instruction("cbz x10, __rt_ob_end_clean_empty");
    emitter.instruction("sub x10, x10, #1");                                    // pop the top level (discard its bytes)
    emitter.instruction("str x10, [x9]");
    emitter.instruction("mov x0, #1");
    emitter.instruction("ret");

    emitter.label("__rt_ob_end_clean_empty");
    emitter.instruction("mov x0, #2");                                          // fd = stderr for the empty-stack notice
    abi::emit_symbol_address(emitter, "x1", "_ob_end_clean_empty_msg");
    emitter.instruction(&format!("mov x2, #{}", crate::codegen::runtime::data::OB_END_CLEAN_EMPTY_MSG.len()));
    emitter.syscall(4);
    emitter.instruction("mov x0, #0");                                          // ob_end_clean() returns false on an empty stack
    emitter.instruction("ret");
}

/// x86_64 (Linux) counterpart of `emit_ob_end_clean`.
fn emit_ob_end_clean_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ob_end_clean ---");
    emitter.label_global("__rt_ob_end_clean");

    abi::emit_symbol_address(emitter, "r8", "_ob_level");
    emitter.instruction("mov r9, QWORD PTR [r8]");
    emitter.instruction("test r9, r9");
    emitter.instruction("jz __rt_ob_end_clean_empty_x86_64");
    emitter.instruction("dec r9");                                              // pop the top level (discard its bytes)
    emitter.instruction("mov QWORD PTR [r8], r9");
    emitter.instruction("mov eax, 1");
    emitter.instruction("ret");

    emitter.label("__rt_ob_end_clean_empty_x86_64");
    abi::emit_symbol_address(emitter, "rsi", "_ob_end_clean_empty_msg");
    emitter.instruction(&format!("mov edx, {}", crate::codegen::runtime::data::OB_END_CLEAN_EMPTY_MSG.len()));
    emitter.instruction("mov edi, 2");                                          // fd = stderr for the empty-stack notice
    emitter.instruction("mov eax, 1");
    emitter.instruction("syscall");
    emitter.instruction("xor eax, eax");                                        // ob_end_clean() returns false on an empty stack
    emitter.instruction("ret");
}

/// Emits `__rt_ob_end_flush`: writes the current top output-buffering level's
/// bytes THROUGH to whatever is below it (an enclosing buffer if one is still
/// active after the pop, otherwise the real syscall/`--web` capture path), then
/// pops the level. Returns `1` (true) on success, `0` (false) plus a PHP-style
/// stderr notice on an empty stack (php -n verified: "Failed to delete and
/// flush buffer. No buffer to delete or flush").
pub fn emit_ob_end_flush(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ob_end_flush_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ob_end_flush ---");
    emitter.label_global("__rt_ob_end_flush");

    // -- minimal frame: the write-through re-enters __rt_stdout_write, which clobbers x30 --
    emitter.instruction("stp x29, x30, [sp, #-16]!");
    emitter.instruction("mov x29, sp");

    abi::emit_symbol_address(emitter, "x9", "_ob_level");
    emitter.instruction("ldr x10, [x9]");
    emitter.instruction("cbz x10, __rt_ob_end_flush_empty");
    emitter.instruction("sub x10, x10, #1");                                    // idx = level - 1; also the new (popped) level
    abi::emit_symbol_address(emitter, "x11", "_ob_offs");
    emitter.instruction("lsl x12, x10, #3");
    emitter.instruction("add x11, x11, x12");
    emitter.instruction("ldr x1, [x11]");                                       // captured level's byte count -> stdout_write len arg
    abi::emit_symbol_address(emitter, "x0", "_ob_bufs");
    emitter.instruction(&format!("lsl x12, x10, #{}", OB_LEVEL_CAP_SHIFT));
    emitter.instruction("add x0, x0, x12");                                     // captured level's buffer base -> stdout_write ptr arg
    emitter.instruction("str x10, [x9]");                                       // pop BEFORE the write-through re-entry
    emitter.instruction("bl __rt_stdout_write");                                // route the popped level's bytes to the new destination
    emitter.instruction("mov x0, #1");
    emitter.instruction("ldp x29, x30, [sp], #16");
    emitter.instruction("ret");

    emitter.label("__rt_ob_end_flush_empty");
    emit_ob_fatal_aarch64_notice(emitter, "_ob_end_flush_empty_msg", crate::codegen::runtime::data::OB_END_FLUSH_EMPTY_MSG.len());
    emitter.instruction("mov x0, #0");                                          // ob_end_flush() returns false on an empty stack
    emitter.instruction("ldp x29, x30, [sp], #16");
    emitter.instruction("ret");
}

/// Writes a stderr notice (not a fatal abort) — shared by the `_empty` branches
/// of `__rt_ob_end_clean`/`__rt_ob_end_flush` on AArch64.
fn emit_ob_fatal_aarch64_notice(emitter: &mut Emitter, msg_symbol: &str, msg_len: usize) {
    emitter.instruction("mov x0, #2");                                          // fd = stderr for the empty-stack notice
    abi::emit_symbol_address(emitter, "x1", msg_symbol);
    emitter.instruction(&format!("mov x2, #{}", msg_len));
    emitter.syscall(4);
}

/// x86_64 (Linux) counterpart of `emit_ob_end_flush`.
fn emit_ob_end_flush_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ob_end_flush ---");
    emitter.label_global("__rt_ob_end_flush");

    emitter.instruction("push rbp");                                            // minimal frame: the write-through re-enters __rt_stdout_write
    emitter.instruction("mov rbp, rsp");

    abi::emit_symbol_address(emitter, "r8", "_ob_level");
    emitter.instruction("mov r9, QWORD PTR [r8]");
    emitter.instruction("test r9, r9");
    emitter.instruction("jz __rt_ob_end_flush_empty_x86_64");
    emitter.instruction("dec r9");                                              // idx = level - 1; also the new (popped) level
    abi::emit_symbol_address(emitter, "r10", "_ob_offs");
    emitter.instruction("mov rsi, QWORD PTR [r10 + r9*8]");                     // captured level's byte count -> stdout_write len arg
    abi::emit_symbol_address(emitter, "rdi", "_ob_bufs");
    emitter.instruction(&format!("mov r11, {}", OB_LEVEL_CAP));
    emitter.instruction("imul r11, r9");                                        // idx * OB_LEVEL_CAP
    emitter.instruction("add rdi, r11");                                        // captured level's buffer base -> stdout_write ptr arg
    emitter.instruction("mov QWORD PTR [r8], r9");                              // pop BEFORE the write-through re-entry
    emitter.instruction("call __rt_stdout_write");                              // route the popped level's bytes to the new destination
    emitter.instruction("mov eax, 1");
    emitter.instruction("pop rbp");
    emitter.instruction("ret");

    emitter.label("__rt_ob_end_flush_empty_x86_64");
    abi::emit_symbol_address(emitter, "rsi", "_ob_end_flush_empty_msg");
    emitter.instruction(&format!("mov edx, {}", crate::codegen::runtime::data::OB_END_FLUSH_EMPTY_MSG.len()));
    emitter.instruction("mov edi, 2");                                          // fd = stderr for the empty-stack notice
    emitter.instruction("mov eax, 1");
    emitter.instruction("syscall");
    emitter.instruction("xor eax, eax");                                        // ob_end_flush() returns false on an empty stack
    emitter.instruction("pop rbp");
    emitter.instruction("ret");
}

/// Emits `__rt_ob_get_level`: returns `_ob_level` (the current output-buffering
/// nesting depth, 0 when no buffer is active) in the int result register.
pub fn emit_ob_get_level(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ob_get_level_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ob_get_level ---");
    emitter.label_global("__rt_ob_get_level");
    abi::emit_symbol_address(emitter, "x9", "_ob_level");
    emitter.instruction("ldr x0, [x9]");                                        // load the current nesting depth as the result
    emitter.instruction("ret");
}

/// x86_64 (Linux) counterpart of `emit_ob_get_level`.
fn emit_ob_get_level_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ob_get_level ---");
    emitter.label_global("__rt_ob_get_level");
    abi::emit_symbol_address(emitter, "r8", "_ob_level");
    emitter.instruction("mov rax, QWORD PTR [r8]");                             // load the current nesting depth as the result
    emitter.instruction("ret");
}

/// Emits `__rt_headers_sent`: returns `_headers_sent` (1 once real output has
/// left the output-buffering stack, 0 otherwise) in the int result register.
/// `_headers_sent` itself is stamped by `__rt_stdout_write`'s real-write path
/// (see `crate::codegen::runtime::io::stdout_write`), not here.
pub fn emit_headers_sent(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_headers_sent_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: headers_sent ---");
    emitter.label_global("__rt_headers_sent");
    abi::emit_symbol_address(emitter, "x9", "_headers_sent");
    emitter.instruction("ldr x0, [x9]");                                        // load the headers-sent flag as the result
    emitter.instruction("ret");
}

/// x86_64 (Linux) counterpart of `emit_headers_sent`.
fn emit_headers_sent_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: headers_sent ---");
    emitter.label_global("__rt_headers_sent");
    abi::emit_symbol_address(emitter, "r8", "_headers_sent");
    emitter.instruction("mov rax, QWORD PTR [r8]");                             // load the headers-sent flag as the result
    emitter.instruction("ret");
}

/// Emits `__rt_ob_incompat_fatal`: writes a caller-supplied message to stderr
/// and exits with code 1. Never returns. Shared tail-call target for every
/// raw-syscall output path that bypasses the `ob_start()` buffer (`printf`/
/// `vprintf`/`var_dump`/`print_r` array-hash walkers — see
/// `emit_ob_incompat_check`): each of those entries jumps here directly
/// (`b`/`jmp`, not `bl`/`call`) instead of returning, so no caller frame ever
/// needs to be restored.
/// Input: AArch64 `x0`=msg ptr `x1`=msg len / x86_64 `rdi`=msg ptr `rsi`=msg len.
pub fn emit_ob_incompat_fatal(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ob_incompat_fatal_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ob_incompat_fatal ---");
    emitter.label_global("__rt_ob_incompat_fatal");
    emitter.instruction("mov x2, x1");                                          // syscall len = message length
    emitter.instruction("mov x1, x0");                                          // syscall buf = message pointer
    emitter.instruction("mov x0, #2");                                          // fd = stderr
    emitter.syscall(4);
    emitter.instruction("mov x0, #1");                                          // exit code 1 for the fatal abort
    emitter.syscall(1);
}

/// x86_64 (Linux) counterpart of `emit_ob_incompat_fatal`.
fn emit_ob_incompat_fatal_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ob_incompat_fatal ---");
    emitter.label_global("__rt_ob_incompat_fatal");
    emitter.instruction("mov rdx, rsi");                                        // syscall len = message length
    emitter.instruction("mov rsi, rdi");                                        // syscall buf = message pointer
    emitter.instruction("mov edi, 2");                                          // fd = stderr
    emitter.instruction("mov eax, 1");                                          // Linux x86_64 syscall 1 = write
    emitter.instruction("syscall");
    emitter.instruction("mov edi, 1");                                          // exit code 1 for the fatal abort
    emitter.instruction("mov eax, 60");                                         // Linux x86_64 syscall 60 = exit
    emitter.instruction("syscall");
}

/// Emits the shared "loud when buffered" entry guard for a raw-syscall output
/// walker that bypasses `__rt_stdout_write` (`ob_start()`'s choke point):
/// loads `_ob_level`; when nonzero, tail-jumps to `__rt_ob_incompat_fatal`
/// with `msg_symbol`/`msg_len` (never returns — the caller's own body is
/// skipped entirely, so no partial output leaks before the fatal); when zero,
/// falls straight through (one load + one compare-and-branch, the ONLY cost
/// paid on the common no-buffer path). Callers emit this as the very FIRST
/// instructions of the walker's entry — before any of the walker's own
/// argument handling — so nothing is written before the check fires.
///
/// `label_prefix` must be UNIQUE per call site (this file has no per-function
/// label counter to draw from, unlike `FunctionContext::next_label` in
/// `codegen_ir`): every guarded walker entry picks its own distinct prefix
/// (e.g. `"vd_arr_int"`, `"pr_hash"`) so the generated skip label never
/// collides with another guarded entry's.
pub fn emit_ob_incompat_check(emitter: &mut Emitter, label_prefix: &str, msg_symbol: &str, msg_len: usize) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ob_incompat_check_x86_64(emitter, label_prefix, msg_symbol, msg_len);
        return;
    }
    abi::emit_symbol_address(emitter, "x9", "_ob_level");
    emitter.instruction("ldr x9, [x9]");                                        // current output-buffering nesting depth
    let skip = format!("__rt_ob_incompat_{}_ok", label_prefix);
    emitter.instruction(&format!("cbz x9, {}", skip));                          // no buffer active: fall straight through
    abi::emit_symbol_address(emitter, "x0", msg_symbol);
    emitter.instruction(&format!("mov x1, #{}", msg_len));                      // message byte length
    emitter.instruction("b __rt_ob_incompat_fatal");                            // buffer active: loud, never silently bypass it
    emitter.label(&skip);
}

/// x86_64 (Linux) counterpart of `emit_ob_incompat_check`.
fn emit_ob_incompat_check_x86_64(emitter: &mut Emitter, label_prefix: &str, msg_symbol: &str, msg_len: usize) {
    abi::emit_symbol_address(emitter, "r8", "_ob_level");
    emitter.instruction("mov r8, QWORD PTR [r8]");                              // current output-buffering nesting depth
    emitter.instruction("test r8, r8");
    let skip = format!("__rt_ob_incompat_{}_ok", label_prefix);
    emitter.instruction(&format!("jz {}", skip));                               // no buffer active: fall straight through
    abi::emit_symbol_address(emitter, "rdi", msg_symbol);
    emitter.instruction(&format!("mov esi, {}", msg_len));                      // message byte length
    emitter.instruction("jmp __rt_ob_incompat_fatal");                          // buffer active: loud, never silently bypass it
    emitter.label(&skip);
}

/// Emits `__rt_ob_get_status`: builds `ob_get_status()`'s associative array for
/// the CURRENT top output-buffering level (elephc supports only the
/// `$full_status` == `false`/omitted shape — see the file-level residual note),
/// or an empty hash when no buffer is active (php -n verified: `array(0){}`,
/// not `false`). No arguments; returns the hash pointer in the int result
/// register (the raw `AssocArray` result codegen expects, like `__rt_getdate`).
///
/// Fields (php -n verified against a plain, callback-free `ob_start()`):
/// `name` = `"default output handler"`, `type` = `0`, `flags` = `112`,
/// `level` = the 0-indexed buffer level, `chunk_size` = `0`, `buffer_size` =
/// elephc's real fixed `OB_LEVEL_CAP` (1 MiB) — an honest value distinct from
/// PHP's own growable-chunk default, since elephc's buffer never reallocates —
/// and `buffer_used` = the level's actual captured byte count.
pub fn emit_ob_get_status(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ob_get_status ---");
    emitter.label_global("__rt_ob_get_status");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("sub sp, sp, #32");                             // frame: [sp]=hash ptr, [sp+16..]=saved x29/x30
            emitter.instruction("stp x29, x30, [sp, #16]");                     // save frame pointer and return address
            emitter.instruction("add x29, sp, #16");                            // set the frame pointer
            emitter.instruction("mov x0, #8");                                  // initial capacity 8 (>= 7 entries, avoids a mid-build realloc)
            emitter.instruction("mov x1, #7");                                  // value type = mixed (int and string values)
            emitter.instruction("bl __rt_hash_new");                            // -> x0 = new hash table
            emitter.instruction("str x0, [sp, #0]");                            // save the hash table pointer
            abi::emit_symbol_address(emitter, "x9", "_ob_level");
            emitter.instruction("ldr x9, [x9]");                                // current output-buffering nesting depth
            emitter.instruction("cbz x9, __rt_ob_get_status_done");             // no buffer active: return the empty hash as-is
            // -- 'name' => "default output handler" (string) --
            abi::emit_symbol_address(emitter, "x3", "_ob_status_name");
            emitter.instruction(&format!("mov x4, #{}", crate::codegen::runtime::data::OB_STATUS_NAME.len()));
            emitter.instruction("mov x5, #1");                                  // value tag = string
            abi::emit_symbol_address(emitter, "x1", "_ob_status_k_name");
            emitter.instruction("mov x2, #4");                                  // length of "name"
            emitter.instruction("ldr x0, [sp, #0]");                            // reload the hash table pointer
            emitter.instruction("bl __rt_hash_set");
            emitter.instruction("str x0, [sp, #0]");                            // save the (possibly reallocated) hash table
            // -- 'type' => 0 (int) --
            emit_ob_status_int_field_aarch64(emitter, "_ob_status_k_type", 4, 0);
            // -- 'flags' => 112 (int) --
            emit_ob_status_int_field_aarch64(emitter, "_ob_status_k_flags", 5, 112);
            // -- 'level' => 0-indexed level (int) --
            abi::emit_symbol_address(emitter, "x9", "_ob_level");
            emitter.instruction("ldr x3, [x9]");                                // current level
            emitter.instruction("sub x3, x3, #1");                              // 0-indexed
            emitter.instruction("mov x4, #0");                                  // value_hi = 0
            emitter.instruction("mov x5, #0");                                  // value tag = int
            abi::emit_symbol_address(emitter, "x1", "_ob_status_k_level");
            emitter.instruction("mov x2, #5");                                  // length of "level"
            emitter.instruction("ldr x0, [sp, #0]");                            // reload the hash table pointer
            emitter.instruction("bl __rt_hash_set");
            emitter.instruction("str x0, [sp, #0]");                            // save the (possibly reallocated) hash table
            // -- 'chunk_size' => 0 (int) --
            emit_ob_status_int_field_aarch64(emitter, "_ob_status_k_chunk_size", 10, 0);
            // -- 'buffer_size' => OB_LEVEL_CAP (int) --
            emit_ob_status_int_field_aarch64(emitter, "_ob_status_k_buffer_size", 11, OB_LEVEL_CAP as i64);
            // -- 'buffer_used' => this level's captured byte count (int) --
            abi::emit_symbol_address(emitter, "x9", "_ob_level");
            emitter.instruction("ldr x9, [x9]");                                // current level
            emitter.instruction("sub x9, x9, #1");                              // idx = level - 1
            abi::emit_symbol_address(emitter, "x10", "_ob_offs");
            emitter.instruction("lsl x11, x9, #3");
            emitter.instruction("ldr x3, [x10, x11]");                          // this level's captured byte count
            emitter.instruction("mov x4, #0");                                  // value_hi = 0
            emitter.instruction("mov x5, #0");                                  // value tag = int
            abi::emit_symbol_address(emitter, "x1", "_ob_status_k_buffer_used");
            emitter.instruction("mov x2, #11");                                 // length of "buffer_used"
            emitter.instruction("ldr x0, [sp, #0]");                            // reload the hash table pointer
            emitter.instruction("bl __rt_hash_set");
            emitter.instruction("str x0, [sp, #0]");                            // save the (possibly reallocated) hash table

            emitter.label("__rt_ob_get_status_done");
            emitter.instruction("ldr x0, [sp, #0]");                            // return the assoc array (hash pointer) in x0
            emitter.instruction("ldp x29, x30, [sp, #16]");                     // restore frame pointer and return address
            emitter.instruction("add sp, sp, #32");                             // deallocate the frame
            emitter.instruction("ret");
        }
        Arch::X86_64 => {
            emitter.instruction("push rbp");                                    // frame: [rbp-8]=hash ptr
            emitter.instruction("mov rbp, rsp");                                // establish the frame pointer
            emitter.instruction("sub rsp, 16");                                 // reserve the local slot (16-aligned)
            emitter.instruction("mov rdi, 8");                                  // initial capacity 8 (>= 7 entries, avoids a mid-build realloc)
            emitter.instruction("mov rsi, 7");                                  // value type = mixed (int and string values)
            emitter.instruction("call __rt_hash_new");                          // -> rax = new hash table
            emitter.instruction("mov QWORD PTR [rbp - 8], rax");                // save the hash table pointer
            abi::emit_symbol_address(emitter, "r8", "_ob_level");
            emitter.instruction("mov r8, QWORD PTR [r8]");                      // current output-buffering nesting depth
            emitter.instruction("test r8, r8");
            emitter.instruction("jz __rt_ob_get_status_done_x86_64");           // no buffer active: return the empty hash as-is
            // -- 'name' => "default output handler" (string) --
            abi::emit_symbol_address(emitter, "rcx", "_ob_status_name");
            emitter.instruction(&format!("mov r8, {}", crate::codegen::runtime::data::OB_STATUS_NAME.len()));
            emitter.instruction("mov r9, 1");                                   // value tag = string
            abi::emit_symbol_address(emitter, "rsi", "_ob_status_k_name");
            emitter.instruction("mov rdx, 4");                                  // length of "name"
            emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                // reload the hash table pointer
            emitter.instruction("call __rt_hash_set");
            emitter.instruction("mov QWORD PTR [rbp - 8], rax");                // save the (possibly reallocated) hash table
            // -- 'type' => 0 (int) --
            emit_ob_status_int_field_x86_64(emitter, "_ob_status_k_type", 4, 0);
            // -- 'flags' => 112 (int) --
            emit_ob_status_int_field_x86_64(emitter, "_ob_status_k_flags", 5, 112);
            // -- 'level' => 0-indexed level (int) --
            abi::emit_symbol_address(emitter, "r8", "_ob_level");
            emitter.instruction("mov rcx, QWORD PTR [r8]");                     // current level
            emitter.instruction("dec rcx");                                     // 0-indexed
            emitter.instruction("mov r8, 0");                                   // value_hi = 0
            emitter.instruction("mov r9, 0");                                   // value tag = int
            abi::emit_symbol_address(emitter, "rsi", "_ob_status_k_level");
            emitter.instruction("mov rdx, 5");                                  // length of "level"
            emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                // reload the hash table pointer
            emitter.instruction("call __rt_hash_set");
            emitter.instruction("mov QWORD PTR [rbp - 8], rax");                // save the (possibly reallocated) hash table
            // -- 'chunk_size' => 0 (int) --
            emit_ob_status_int_field_x86_64(emitter, "_ob_status_k_chunk_size", 10, 0);
            // -- 'buffer_size' => OB_LEVEL_CAP (int) --
            emit_ob_status_int_field_x86_64(emitter, "_ob_status_k_buffer_size", 11, OB_LEVEL_CAP as i64);
            // -- 'buffer_used' => this level's captured byte count (int) --
            abi::emit_symbol_address(emitter, "r8", "_ob_level");
            emitter.instruction("mov r9, QWORD PTR [r8]");                      // current level
            emitter.instruction("dec r9");                                      // idx = level - 1
            abi::emit_symbol_address(emitter, "r10", "_ob_offs");
            emitter.instruction("mov rcx, QWORD PTR [r10 + r9*8]");             // this level's captured byte count
            emitter.instruction("mov r8, 0");                                   // value_hi = 0
            emitter.instruction("mov r9, 0");                                   // value tag = int
            abi::emit_symbol_address(emitter, "rsi", "_ob_status_k_buffer_used");
            emitter.instruction("mov rdx, 11");                                 // length of "buffer_used"
            emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                // reload the hash table pointer
            emitter.instruction("call __rt_hash_set");
            emitter.instruction("mov QWORD PTR [rbp - 8], rax");                // save the (possibly reallocated) hash table

            emitter.label("__rt_ob_get_status_done_x86_64");
            emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                // return the assoc array (hash pointer) in rax
            emitter.instruction("add rsp, 16");                                 // deallocate the local slot
            emitter.instruction("pop rbp");                                     // restore the caller frame pointer
            emitter.instruction("ret");
        }
    }
}

/// Appends one constant-int status field (AArch64): reloads the hash pointer
/// from `[sp, #0]`, calls `__rt_hash_set`, and saves the (possibly reallocated)
/// result back to `[sp, #0]`. Shared by the `type`/`flags`/`chunk_size`/
/// `buffer_size` fields, which are all fixed constants for elephc's one
/// supported (callback-free) buffer kind.
fn emit_ob_status_int_field_aarch64(emitter: &mut Emitter, key_symbol: &str, key_len: i64, value: i64) {
    emitter.instruction(&format!("mov x3, #{}", value));                        // constant field value
    emitter.instruction("mov x4, #0");                                          // value_hi = 0
    emitter.instruction("mov x5, #0");                                          // value tag = int
    abi::emit_symbol_address(emitter, "x1", key_symbol);
    emitter.instruction(&format!("mov x2, #{}", key_len));                      // key length
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the hash table pointer
    emitter.instruction("bl __rt_hash_set");
    emitter.instruction("str x0, [sp, #0]");                                    // save the (possibly reallocated) hash table
}

/// x86_64 (Linux) counterpart of `emit_ob_status_int_field_aarch64`.
fn emit_ob_status_int_field_x86_64(emitter: &mut Emitter, key_symbol: &str, key_len: i64, value: i64) {
    emitter.instruction(&format!("mov rcx, {}", value));                        // constant field value
    emitter.instruction("mov r8, 0");                                           // value_hi = 0
    emitter.instruction("mov r9, 0");                                           // value tag = int
    abi::emit_symbol_address(emitter, "rsi", key_symbol);
    emitter.instruction(&format!("mov rdx, {}", key_len));                      // key length
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the hash table pointer
    emitter.instruction("call __rt_hash_set");
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the (possibly reallocated) hash table
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::platform::{Platform, Target};

    /// Renders every `__rt_ob_*`/`__rt_headers_sent` helper for one target.
    fn render(arch: Arch) -> String {
        let platform = if arch == Arch::X86_64 { Platform::Linux } else { Platform::MacOS };
        let mut emitter = Emitter::new(Target::new(platform, arch));
        emit_ob_start(&mut emitter);
        emit_ob_append(&mut emitter);
        emit_ob_peek_contents(&mut emitter);
        emit_ob_pop(&mut emitter);
        emit_ob_end_clean(&mut emitter);
        emit_ob_end_flush(&mut emitter);
        emit_ob_get_level(&mut emitter);
        emit_ob_get_status(&mut emitter);
        emit_headers_sent(&mut emitter);
        emitter.output()
    }

    /// Verifies every global label is emitted for every supported target.
    #[test]
    fn emits_global_labels_for_all_targets() {
        for arch in [Arch::AArch64, Arch::X86_64] {
            let asm = render(arch);
            for label in [
                "__rt_ob_start",
                "__rt_ob_append",
                "__rt_ob_peek_contents",
                "__rt_ob_pop",
                "__rt_ob_end_clean",
                "__rt_ob_end_flush",
                "__rt_ob_get_level",
                "__rt_ob_get_status",
                "__rt_headers_sent",
            ] {
                assert!(
                    asm.contains(&format!(".globl {}\n", label)),
                    "missing global label {} for {:?}",
                    label,
                    arch
                );
            }
        }
    }

    /// Best-effort x86_64 assembler syntax check via `clang` (skipped when
    /// `clang` isn't on PATH) — catches Linux x86_64 assembly mistakes
    /// locally without needing the Linux Docker cross-compile scripts.
    /// Prepends the same `.intel_syntax noprefix`/`.text` prelude the real
    /// pipeline emits once per file (`crate::codegen::emit::Emitter`'s text
    /// prelude), since this test renders the helpers standalone.
    #[test]
    fn x86_64_output_assembles_with_clang() {
        let asm = format!(".intel_syntax noprefix\n.text\n{}", render(Arch::X86_64));
        let Ok(mut child) = std::process::Command::new("clang")
            .args(["-target", "x86_64-linux-gnu", "-c", "-x", "assembler", "-o", "/dev/null", "-"])
            .stdin(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        else {
            eprintln!("skipping x86_64_output_assembles_with_clang: clang not available");
            return;
        };
        use std::io::Write;
        child
            .stdin
            .take()
            .expect("piped stdin")
            .write_all(asm.as_bytes())
            .expect("write asm to clang stdin");
        let output = child.wait_with_output().expect("wait for clang");
        assert!(
            output.status.success(),
            "clang failed to assemble x86_64 output:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
