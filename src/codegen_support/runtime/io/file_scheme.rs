//! Purpose:
//! Emits `__rt_path_cstr` and `__rt_path_cstr2`, the path-only front ends to `__rt_cstr` that
//! strip php's `file://` URL prefix before the path reaches a syscall.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//! - Every runtime helper whose argument is a FILESYSTEM PATH: the open, stat, directory and
//!   metadata families.
//!
//! Key details:
//! - `stream_get_wrappers()` has always listed `file`, and NOTHING honoured it: MEASURED, twelve
//!   operations in a row — `file_get_contents`, `fopen`, `file_exists`, `filesize`, `is_file`,
//!   `file`, `copy`, `file_put_contents`, `is_dir`, `mkdir`, `unlink`, `rename` — every one
//!   answered `false` for a URL php reads without complaint.
//! - THE RULE, measured on `php -n` 8.5.6:
//!   - `file://` matches case-insensitively (`FILE://`, `FiLe://` both work), and needs exactly
//!     `://`: `file:/abs` and `file:abs` are not URLs and stay verbatim.
//!   - What follows is an authority, then the path. The authority must be EMPTY (the path starts
//!     at the `/`) or exactly `localhost`, case-insensitively. `file://example.com/abs` is
//!     refused, and so is `file://u.txt` — whose "host" is `u.txt` and whose path is empty.
//!   - Extra slashes belong to the path: `file:////abs` opens `//abs`, which POSIX collapses.
//! - NOT EVERY PATH BUILTIN. php routes these through its plain-files WRAPPER, and the calls that
//!   go straight to libc instead do not see the URL at all: `realpath()`, `readlink()`, the LINK
//!   argument of `symlink()`, `glob()`, `disk_free_space()` and `chdir()` all answer `false` for
//!   a `file://` URL in php. Each of those was measured; they keep calling `__rt_cstr` directly.
//! - The prefix is dropped by moving the POINTER, never by copying: the caller's string is
//!   untouched, so a diagnostic that names the path still names the URL the program wrote —
//!   which is what php prints, `Failed to open stream` and all.
//! - REGISTER DISCIPLINE. The strip may only touch what `__rt_cstr` itself already clobbers, or a
//!   caller that survives `bl __rt_cstr` today would stop surviving the front end that replaces
//!   it. That is why `localhost` is compared against IMMEDIATES rather than against a data
//!   symbol: a symbol address needs a register to hold it, and on AArch64 `abi` resolves one
//!   through `x9` — the very scratch this uses. The nine comparisons are the cheaper price.
//! - The two entry points repeat the strip rather than share it through a flag register, for the
//!   same reason: the flag would be a sixth live value with nowhere safe to live.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Byte length of the `file://` prefix.
const FILE_SCHEME_LEN: i64 = 7;

/// Byte length of the only authority php accepts besides an empty one.
const LOCALHOST_LEN: i64 = 9;

/// The authority php accepts, one byte per comparison.
const LOCALHOST: &[u8] = b"localhost";

/// Emits `__rt_path_cstr` and `__rt_path_cstr2`.
///
/// Same contract as `__rt_cstr`/`__rt_cstr2` — AArch64 `x1` = pointer, `x2` = length, answer in
/// `x0`; x86_64 `rax` = pointer, `rdx` = length, answer in `rax` — with a leading `file://` URL
/// reduced to the path it names first.
///
/// Both end in a tail jump, because `__rt_cstr` is a leaf: there is no frame to build and no link
/// register to save, which is what lets these sit in front of helpers that have not set one up.
pub fn emit_path_cstr(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.blank();
            emitter.comment("--- runtime: path_cstr ---");
            emitter.label_global("__rt_path_cstr");
            emit_strip_aarch64(emitter, "one");
            emitter.instruction("b __rt_cstr");                                 // tail jump: __rt_cstr is a leaf

            emitter.blank();
            emitter.comment("--- runtime: path_cstr2 ---");
            emitter.label_global("__rt_path_cstr2");
            emit_strip_aarch64(emitter, "two");
            emitter.instruction("b __rt_cstr2");                                // tail jump: __rt_cstr2 is a leaf
        }
        Arch::X86_64 => {
            emitter.blank();
            emitter.comment("--- runtime: path_cstr ---");
            emitter.label_global("__rt_path_cstr");
            emit_strip_x86_64(emitter, "one");
            emitter.instruction("jmp __rt_cstr");                               // tail jump: __rt_cstr is a leaf

            emitter.blank();
            emitter.comment("--- runtime: path_cstr2 ---");
            emitter.label_global("__rt_path_cstr2");
            emit_strip_x86_64(emitter, "two");
            emitter.instruction("jmp __rt_cstr2");                              // tail jump: __rt_cstr2 is a leaf
        }
    }
}

/// Reduces `x1`/`x2` from a `file://` URL to the path it names, or leaves them alone.
fn emit_strip_aarch64(emitter: &mut Emitter, tag: &str) {
    let done = format!("__rt_path_cstr_{tag}_done");
    let take = format!("__rt_path_cstr_{tag}_take");

    emitter.instruction(&format!("cmp x2, #{FILE_SCHEME_LEN}"));
    emitter.instruction(&format!("b.lt {done}"));                               // too short to carry a scheme

    // -- `file://`, case-insensitively --
    for (offset, letter) in [(0, b'f'), (1, b'i'), (2, b'l'), (3, b'e')] {
        emitter.instruction(&format!("ldrb w9, [x1, #{offset}]"));
        emitter.instruction("orr w9, w9, #0x20");                               // fold the letter to lower case
        emitter.instruction(&format!("cmp w9, #{letter}"));
        emitter.instruction(&format!("b.ne {done}"));
    }
    for (offset, byte) in [(4, b':'), (5, b'/'), (6, b'/')] {
        emitter.instruction(&format!("ldrb w9, [x1, #{offset}]"));
        emitter.instruction(&format!("cmp w9, #{byte}"));
        emitter.instruction(&format!("b.ne {done}"));                           // `file:/abs` is a filename, not a URL
    }

    emitter.instruction(&format!("add x10, x1, #{FILE_SCHEME_LEN}"));           // the authority starts here
    emitter.instruction(&format!("sub x11, x2, #{FILE_SCHEME_LEN}"));
    emitter.instruction(&format!("cbz x11, {done}"));                           // nothing after the scheme names nothing
    emitter.instruction("ldrb w9, [x10]");
    emitter.instruction("cmp w9, #47");                                         // an empty authority: the path starts at this '/'
    emitter.instruction(&format!("b.eq {take}"));

    // -- the only other authority php accepts is `localhost`, and a path must follow it --
    emitter.instruction(&format!("cmp x11, #{}", LOCALHOST_LEN + 1));
    emitter.instruction(&format!("b.lt {done}"));
    for (offset, letter) in LOCALHOST.iter().enumerate() {
        emitter.instruction(&format!("ldrb w9, [x10, #{offset}]"));
        emitter.instruction("orr w9, w9, #0x20");                               // php matches the host case-insensitively
        emitter.instruction(&format!("cmp w9, #{letter}"));
        emitter.instruction(&format!("b.ne {done}"));                           // any other host is refused, URL and all
    }
    emitter.instruction(&format!("ldrb w9, [x10, #{LOCALHOST_LEN}]"));
    emitter.instruction("cmp w9, #47");                                         // `file://localhost` with no path names nothing
    emitter.instruction(&format!("b.ne {done}"));
    emitter.instruction(&format!("add x10, x10, #{LOCALHOST_LEN}"));
    emitter.instruction(&format!("sub x11, x11, #{LOCALHOST_LEN}"));

    emitter.label(&take);
    emitter.instruction("mov x1, x10");                                         // the path the URL names
    emitter.instruction("mov x2, x11");

    emitter.label(&done);
}

/// The x86_64 mirror: `rax` carries the pointer and `rdx` the length.
fn emit_strip_x86_64(emitter: &mut Emitter, tag: &str) {
    let done = format!("__rt_path_cstr_{tag}_done_x86");
    let take = format!("__rt_path_cstr_{tag}_take_x86");

    emitter.instruction(&format!("cmp rdx, {FILE_SCHEME_LEN}"));
    emitter.instruction(&format!("jl {done}"));                                 // too short to carry a scheme

    for (offset, letter) in [(0, b'f'), (1, b'i'), (2, b'l'), (3, b'e')] {
        emitter.instruction(&format!("movzx r8d, BYTE PTR [rax + {offset}]"));
        emitter.instruction("or r8d, 0x20");                                    // fold the letter to lower case
        emitter.instruction(&format!("cmp r8d, {letter}"));
        emitter.instruction(&format!("jne {done}"));
    }
    for (offset, byte) in [(4, b':'), (5, b'/'), (6, b'/')] {
        emitter.instruction(&format!("movzx r8d, BYTE PTR [rax + {offset}]"));
        emitter.instruction(&format!("cmp r8d, {byte}"));
        emitter.instruction(&format!("jne {done}"));                            // `file:/abs` is a filename, not a URL
    }

    emitter.instruction("mov r9, rax");
    emitter.instruction(&format!("add r9, {FILE_SCHEME_LEN}"));                 // the authority starts here
    emitter.instruction("mov r10, rdx");
    emitter.instruction(&format!("sub r10, {FILE_SCHEME_LEN}"));
    emitter.instruction("test r10, r10");
    emitter.instruction(&format!("jz {done}"));                                 // nothing after the scheme names nothing
    emitter.instruction("movzx r8d, BYTE PTR [r9]");
    emitter.instruction("cmp r8d, 47");                                         // an empty authority: the path starts at this '/'
    emitter.instruction(&format!("je {take}"));

    emitter.instruction(&format!("cmp r10, {}", LOCALHOST_LEN + 1));
    emitter.instruction(&format!("jl {done}"));
    for (offset, letter) in LOCALHOST.iter().enumerate() {
        emitter.instruction(&format!("movzx r8d, BYTE PTR [r9 + {offset}]"));
        emitter.instruction("or r8d, 0x20");                                    // php matches the host case-insensitively
        emitter.instruction(&format!("cmp r8d, {letter}"));
        emitter.instruction(&format!("jne {done}"));                            // any other host is refused, URL and all
    }
    emitter.instruction(&format!("movzx r8d, BYTE PTR [r9 + {LOCALHOST_LEN}]"));
    emitter.instruction("cmp r8d, 47");                                         // `file://localhost` with no path names nothing
    emitter.instruction(&format!("jne {done}"));
    emitter.instruction(&format!("add r9, {LOCALHOST_LEN}"));
    emitter.instruction(&format!("sub r10, {LOCALHOST_LEN}"));

    emitter.label(&take);
    emitter.instruction("mov rax, r9");                                         // the path the URL names
    emitter.instruction("mov rdx, r10");

    emitter.label(&done);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::{Platform, Target};

    /// The strip may only touch what `__rt_cstr` already clobbers.
    ///
    /// A caller that survives `bl __rt_cstr` today must survive the front end that replaces it,
    /// and the only thing standing between the two is this register list. `__rt_cstr` uses
    /// `x9`-`x12` on AArch64 and `r8`-`r11`/`rcx` on x86_64; anything wider would corrupt a
    /// caller's live state at a call site that used to be safe.
    #[test]
    fn the_strip_stays_inside_the_scratch_cstr_already_clobbers() {
        let mut emitter = Emitter::new(Target::new(Platform::MacOS, Arch::AArch64));
        emit_path_cstr(&mut emitter);
        // The case-folding mask is written `0x20`, which spells a register name inside a
        // substring search; the immediate is removed before the register names are looked for.
        let asm = emitter.output().replace("0x20", "<mask>");
        for reg in ["x13", "x14", "x15", "x16", "x17", "x19", "x20"] {
            assert!(!asm.contains(reg), "the AArch64 strip must not touch {reg}");
        }

        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_path_cstr(&mut emitter);
        let asm = emitter.output().replace("0x20", "<mask>");
        for reg in ["r12", "r13", "r14", "r15", "rbx", "rsi", "rdi"] {
            assert!(!asm.contains(reg), "the x86_64 strip must not touch {reg}");
        }
    }

    /// Both arms end in a TAIL JUMP, because `__rt_cstr` is a leaf with no frame of its own.
    #[test]
    fn both_arms_tail_jump_into_the_cstr_they_front() {
        let mut emitter = Emitter::new(Target::new(Platform::MacOS, Arch::AArch64));
        emit_path_cstr(&mut emitter);
        let asm = emitter.output();
        assert!(asm.contains("b __rt_cstr\n"), "must tail-jump to __rt_cstr");
        assert!(asm.contains("b __rt_cstr2\n"), "must tail-jump to __rt_cstr2");
        assert!(
            !asm.contains("bl __rt_cstr"),
            "a call would need a link register this has not saved"
        );

        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_path_cstr(&mut emitter);
        let asm = emitter.output();
        assert!(asm.contains("jmp __rt_cstr\n"), "must tail-jump to __rt_cstr");
        assert!(asm.contains("jmp __rt_cstr2\n"), "must tail-jump to __rt_cstr2");
        assert!(
            !asm.contains("call __rt_cstr"),
            "a call would push a return address the leaf never pops"
        );
    }

    /// The URL is reduced by moving the pointer, never by copying it.
    #[test]
    fn the_prefix_is_dropped_without_allocating() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
        ] {
            let mut emitter = Emitter::new(target);
            emit_path_cstr(&mut emitter);
            let asm = emitter.output();
            assert!(
                !asm.contains("__rt_heap_alloc") && !asm.contains("memcpy"),
                "the path front end must not allocate or copy"
            );
        }
    }
}
