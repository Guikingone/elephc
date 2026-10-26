//! Purpose:
//! Emits print_r capture writers and transfers complete buffers to owned PHP strings.
//! Keeps writers target-aware while each capture owns its storage.
//!
//! Called from:
//! - The platform runtime emitter and print_r lowering/value walkers.
//! - `__rt_stdout_write` delegates writes while a capture is active.
//!
//! Key details:
//! - Active buffer pointer, capacity, length and mode are separate globals.
//! - Appending grows a string allocation; finish transfers its sole owner.
//! - Empty captures still return an independently owned empty string.

use crate::codegen_support::abi;
use crate::codegen_support::{emit::Emitter, platform::Arch};

mod growth;

pub use growth::emit_pr_append;

/// Emits `__rt_pr_write`: the print_r walker terminal-write indirection.
/// Branches on `_print_r_mode`: 1 → append to the capture buffer via
/// `__rt_pr_append`; 0 → the plain write path, which itself appends to the top
/// output buffer via `__rt_ob_append` while `_ob_level` is non-zero and only
/// falls through to the raw `write(1, buf, len)` syscall otherwise.
///
/// Inputs match the walker's pre-syscall register layout:
/// AArch64 x1=buf, x2=len / x86_64 rsi=buf, rdx=len. No result.
pub fn emit_pr_write(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_pr_write_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: pr_write ---");
    emitter.label_global("__rt_pr_write");

    // -- set up a frame so the append branch can call __rt_pr_append --
    emitter.instruction("stp x29, x30, [sp, #-16]!");                           // save frame pointer and return address (the append branch clobbers x30)
    emitter.instruction("mov x29, sp");                                         // establish a frame pointer for the call

    // -- branch on the capture-mode flag --
    abi::emit_symbol_address(emitter, "x9", "_print_r_mode");                   // materialize the address of the capture-mode flag
    emitter.instruction("ldr x9, [x9]");                                        // load the capture-mode flag
    emitter.instruction("cbz x9, __rt_pr_write_syscall");                       // mode 0 → plain write syscall
    emitter.instruction("mov x0, x1");                                          // __rt_pr_append buf arg = incoming buf pointer
    emitter.instruction("mov x1, x2");                                          // __rt_pr_append len arg = incoming length
    emitter.instruction("bl __rt_pr_append");                                   // append the bytes to the capture buffer
    emitter.instruction("b __rt_pr_write_done");                                // skip the syscall path

    // -- plain write path: while the ob_* stack is non-empty, append to the top
    //    output buffer instead of issuing the write(1, …) syscall so print_r's
    //    structural output is captured like every other terminal write. --
    emitter.label("__rt_pr_write_syscall");
    abi::emit_symbol_address(emitter, "x9", "_ob_in_handler");                  // materialize the address of the in-handler flag
    emitter.instruction("ldr x9, [x9]");                                        // load the in-handler flag
    emitter.instruction("cbnz x9, __rt_pr_write_done");                         // inside a handler — discard the bytes entirely
    abi::emit_symbol_address(emitter, "x9", "_ob_level");                       // materialize the address of the output-buffer stack depth
    emitter.instruction("ldr x9, [x9]");                                        // load the output-buffer stack depth
    emitter.instruction("cbz x9, __rt_pr_write_raw");                           // no active output buffer — take the raw syscall path
    emitter.instruction("mov x0, x1");                                          // __rt_ob_append buf arg = incoming buf pointer
    emitter.instruction("mov x1, x2");                                          // __rt_ob_append len arg = incoming length
    emitter.instruction("bl __rt_ob_append");                                   // append the bytes to the top output buffer
    emitter.instruction("b __rt_pr_write_done");                                // capture handled the bytes — skip the syscall path
    emitter.label("__rt_pr_write_raw");
    emitter.instruction("mov x0, #1");                                          // syscall fd = stdout (buf already in x1, len already in x2)
    emitter.syscall(4);                                                         // write the bytes to stdout

    emitter.label("__rt_pr_write_done");
    emitter.instruction("ldp x29, x30, [sp], #16");                             // restore frame pointer and return address
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits the Linux x86_64 variant of `__rt_pr_write`.
fn emit_pr_write_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: pr_write ---");
    emitter.label_global("__rt_pr_write");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer and align rsp for the append-branch call
    emitter.instruction("mov rbp, rsp");                                        // establish a frame base

    abi::emit_symbol_address(emitter, "r11", "_print_r_mode");                  // materialize the address of the capture-mode flag
    emitter.instruction("mov r11, QWORD PTR [r11]");                            // load the capture-mode flag
    emitter.instruction("test r11, r11");                                       // is return-mode capture enabled?
    emitter.instruction("jz __rt_pr_write_syscall");                            // mode 0 → plain write syscall
    emitter.instruction("mov rdi, rsi");                                        // __rt_pr_append buf arg = incoming buf pointer
    emitter.instruction("mov rsi, rdx");                                        // __rt_pr_append len arg = incoming length
    emitter.instruction("call __rt_pr_append");                                 // append the bytes to the capture buffer
    emitter.instruction("jmp __rt_pr_write_done");                              // skip the syscall path

    // -- plain write path: while the ob_* stack is non-empty, append to the top
    //    output buffer instead of issuing the write(1, …) syscall so print_r's
    //    structural output is captured like every other terminal write. --
    emitter.label("__rt_pr_write_syscall");
    abi::emit_symbol_address(emitter, "r11", "_ob_in_handler");                 // materialize the address of the in-handler flag
    emitter.instruction("mov r11, QWORD PTR [r11]");                            // load the in-handler flag
    emitter.instruction("test r11, r11");                                       // is a user output handler running?
    emitter.instruction("jnz __rt_pr_write_done");                              // inside a handler — discard the bytes entirely
    abi::emit_symbol_address(emitter, "r11", "_ob_level");                      // materialize the address of the output-buffer stack depth
    emitter.instruction("mov r11, QWORD PTR [r11]");                            // load the output-buffer stack depth
    emitter.instruction("test r11, r11");                                       // is any output buffer active?
    emitter.instruction("jz __rt_pr_write_raw_x86");                            // no active output buffer — take the raw syscall path
    emitter.instruction("mov rdi, rsi");                                        // __rt_ob_append buf arg = incoming buf pointer
    emitter.instruction("mov rsi, rdx");                                        // __rt_ob_append len arg = incoming length
    emitter.instruction("call __rt_ob_append");                                 // append the bytes to the top output buffer
    emitter.instruction("jmp __rt_pr_write_done");                              // capture handled the bytes — skip the syscall path
    emitter.label("__rt_pr_write_raw_x86");
    emitter.instruction("mov rdi, 1");                                          // syscall fd = stdout
    // buf is already in rsi; len is already in rdx
    emitter.instruction("mov eax, 1");                                          // Linux x86_64 syscall 1 = write
    emitter.instruction("syscall");                                             // write the bytes to stdout

    emitter.label("__rt_pr_write_done");
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to caller
}

/// Adopts the complete capture allocation as the returned string and clears active state.
pub fn emit_pr_finish(emitter: &mut Emitter) {
    emitter.blank();
    emitter.label_global("__rt_pr_finish");
    abi::emit_frame_prologue(emitter, 16);
    let result = abi::int_result_reg(emitter);
    abi::emit_load_symbol_to_reg(emitter, result, "_print_r_buf", 0);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_pr_finish_empty");
    let (pointer, length) = abi::string_result_regs(emitter);
    abi::emit_reg_move(emitter, pointer, result);
    abi::emit_load_symbol_to_reg(emitter, length, "_print_r_off", 0);
    abi::emit_jump(emitter, "__rt_pr_finish_reset");
    emitter.label("__rt_pr_finish_empty");
    abi::emit_symbol_address(emitter, pointer, "_pr_nl");
    abi::emit_load_int_immediate(emitter, length, 0);
    abi::emit_call_label(emitter, "__rt_str_persist");
    emitter.label("__rt_pr_finish_reset");
    for symbol in ["_print_r_buf", "_print_r_cap", "_print_r_off", "_print_r_mode"] {
        abi::emit_store_zero_to_symbol(emitter, symbol, 0);
    }
    abi::emit_frame_restore(emitter, 16);
    abi::emit_return(emitter);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::{Arch, Platform, Target};

    /// Renders all three print_r return-mode helpers for one target.
    fn render(platform: Platform, arch: Arch) -> String {
        let mut emitter = Emitter::new(Target::new(platform, arch));
        emit_pr_append(&mut emitter);
        emit_pr_write(&mut emitter);
        emit_pr_finish(&mut emitter);
        emitter.output()
    }

    /// Verifies every target exports the three helper labels.
    #[test]
    fn emits_global_labels_for_all_targets() {
        for (platform, arch) in [
            (Platform::MacOS, Arch::AArch64),
            (Platform::Linux, Arch::AArch64),
            (Platform::Linux, Arch::X86_64),
        ] {
            let asm = render(platform, arch);
            assert!(
                asm.contains(".globl __rt_pr_append\n"),
                "missing __rt_pr_append label for {:?}/{:?}",
                platform,
                arch
            );
            assert!(
                asm.contains(".globl __rt_pr_write\n"),
                "missing __rt_pr_write label for {:?}/{:?}",
                platform,
                arch
            );
            assert!(
                asm.contains(".globl __rt_pr_finish\n"),
                "missing __rt_pr_finish label for {:?}/{:?}",
                platform,
                arch
            );
        }
    }

    /// Verifies the walker indirection branches on the capture-mode flag and
    /// references the append helper on every target.
    #[test]
    fn pr_write_branches_on_mode_and_calls_append() {
        let mac = render(Platform::MacOS, Arch::AArch64);
        assert!(mac.contains("_print_r_mode"));
        assert!(mac.contains("bl __rt_pr_append"));
        assert!(mac.contains("__rt_pr_write_syscall"));

        let linux_x86 = render(Platform::Linux, Arch::X86_64);
        assert!(linux_x86.contains("_print_r_mode"));
        assert!(linux_x86.contains("call __rt_pr_append"));
        assert!(linux_x86.contains("__rt_pr_write_syscall"));
    }

    /// Verifies finish can create an empty owned result and resets the active state.
    #[test]
    fn pr_finish_returns_owned_string_and_resets_state() {
        let mac = render(Platform::MacOS, Arch::AArch64);
        assert!(mac.contains("bl __rt_str_persist"));
        assert!(mac.contains("_print_r_off"));
        assert!(mac.contains("_print_r_buf"));

        let linux_x86 = render(Platform::Linux, Arch::X86_64);
        assert!(linux_x86.contains("call __rt_str_persist"));
        assert!(linux_x86.contains("_print_r_off"));
        assert!(linux_x86.contains("_print_r_buf"));
    }
}
