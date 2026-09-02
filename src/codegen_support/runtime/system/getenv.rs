//! Purpose:
//! Emits the named and full-process environment lookup runtime helpers.
//! Keeps PHP builtin semantics, libc/syscall boundaries, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//!
//! Key details:
//! - Named lookup converts PHP strings to C strings. A MISSING variable returns a
//!   null pointer and a PRESENT one returns an owned heap copy of its value —
//!   the two are different answers, and only a null pointer means "not set".
//!   libc hands back a pointer into the environment block, which the caller must
//!   not own, so the found path persists before returning.
//! - Full lookup (`__rt_getenv_all`, the no-argument `getenv()`) splits each process entry at
//!   its first `=` and copies keys and values into a fresh hash.

use crate::codegen_support::{
    abi,
    emit::Emitter,
    platform::{Arch, Platform},
};

/// Runtime value tag for string entries stored in associative arrays.
const STR_TAG: i64 = 1;

/// Emits `__rt_getenv` helper for ARM64 targets (macOS/Linux).
///
/// Converts a PHP string (name ptr in x1, name len in x2) to a C string via
/// `__rt_cstr`, calls libc `getenv`, and returns the value as an owned PHP
/// string (ptr in x1, len in x2), or a null pointer (x1=0, x2=0) when the
/// variable is not set.
///
/// The null pointer is the whole point: PHP's `getenv` answers `false` for a
/// variable that is not set and `""` for one set to the empty string, and those
/// are the two cases the caller has to tell apart. libc already distinguishes
/// them — a missing name gives NULL, an empty value gives a valid pointer to a
/// zero-length string — so the information is here; it is `__rt_str_persist`
/// that carries it, since it gives a zero-length string an owned block rather
/// than a null one.
pub fn emit_getenv(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_getenv_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: getenv ---");
    emitter.label_global("__rt_getenv");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #32");                                     // allocate 32 bytes on the stack
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // set new frame pointer

    // -- null-terminate the name string --
    emitter.instruction("bl __rt_cstr");                                        // convert to C string → x0=null-terminated ptr

    // -- call libc getenv --
    emitter.bl_c("getenv");                                          // getenv(name) → x0=value ptr or NULL

    // -- check for NULL return --
    emitter.instruction("cbz x0, __rt_getenv_unset");                           // libc says NULL only for a name that is not set

    // -- scan for null terminator to compute length --
    emitter.instruction("mov x1, x0");                                          // x1 = value ptr (start)
    emitter.instruction("mov x2, #0");                                          // x2 = length counter
    emitter.label("__rt_getenv_len");
    emitter.instruction("ldrb w9, [x0, x2]");                                   // load byte at offset x2
    emitter.instruction("cbz w9, __rt_getenv_persist");                         // if null terminator, done counting
    emitter.instruction("add x2, x2, #1");                                      // increment length
    emitter.instruction("b __rt_getenv_len");                                   // continue scanning

    // -- copy the value out of the environment block, which the caller must not own --
    //
    // The copy is required by the CONTRACT, not by an observed crash: the result
    // is boxed as an OWNED string, and anything that later classifies that
    // payload reads its heap header at `[ptr - 8]` — eight bytes before a
    // string the allocator never handed out. Removing this does not fail any
    // test on a host where a foreign free is range-rejected, which is exactly
    // why the reason is written down here instead.
    emitter.label("__rt_getenv_persist");
    emitter.instruction("bl __rt_str_persist");                                 // x1/x2 = owned heap copy, non-null even at length zero
    emitter.instruction("b __rt_getenv_done");                                  // skip the not-found path after persisting a real value

    // -- a null pointer, not an empty string: the variable is NOT SET --
    emitter.label("__rt_getenv_unset");
    emitter.instruction("mov x1, #0");                                          // null pointer: the caller boxes this as PHP false
    emitter.instruction("mov x2, #0");                                          // no length to report for a variable that is not set

    // -- clean up and return --
    emitter.label("__rt_getenv_done");
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits `__rt_getenv` helper for x86_64 Linux targets.
///
/// Converts a PHP string (name in rdi via `__rt_cstr`) to a null-terminated C
/// string, calls libc `getenv`, and returns the value as an owned PHP string
/// (rax=ptr, rdx=len), or a null pointer (rax=0, rdx=0) when the variable is
/// not set — the same two answers as the ARM64 helper above, and for the same
/// reason. Uses the System V AMD64 ABI for register conventions and frame
/// layout.
fn emit_getenv_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: getenv ---");
    emitter.label_global("__rt_getenv");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer while the getenv helper performs nested libc calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the x86_64 getenv helper

    abi::emit_call_label(emitter, "__rt_cstr");                                 // convert the elephc string result regs into a null-terminated C string in the scratch buffer
    emitter.instruction("mov rdi, rax");                                        // pass the null-terminated environment variable name in the SysV first-argument register
    emitter.bl_c("getenv");                                                     // getenv(name) → rax=value ptr or NULL

    emitter.instruction("test rax, rax");                                       // did libc return a real environment-value pointer?
    emitter.instruction("je __rt_getenv_unset");                                // a name that is not set is not a name set to ""

    emitter.instruction("mov r8, rax");                                         // preserve the start of the returned environment string for the final PHP string pointer result
    emitter.instruction("mov rdx, 0");                                          // seed the returned PHP string length counter at zero bytes
    emitter.label("__rt_getenv_len");
    emitter.instruction("mov cl, BYTE PTR [r8 + rdx]");                         // load the next byte from the returned C string while measuring its length
    emitter.instruction("test cl, cl");                                         // did we reach the terminating C null byte?
    emitter.instruction("je __rt_getenv_done");                                 // stop scanning once the full environment string length is known; the done path persists
    emitter.instruction("add rdx, 1");                                          // advance the returned PHP string length by one byte
    emitter.instruction("jmp __rt_getenv_len");                                 // continue scanning until the C string terminator is found

    emitter.label("__rt_getenv_unset");
    emitter.instruction("mov rax, 0");                                          // null pointer: the caller boxes this as PHP false
    emitter.instruction("mov rdx, 0");                                          // no length to report for a variable that is not set
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning the not-set answer
    emitter.instruction("ret");                                                 // return the null pointer the caller boxes as PHP false

    emitter.label("__rt_getenv_done");
    emitter.instruction("mov rax, r8");                                         // move the environment-block pointer into str_persist's source register
    abi::emit_call_label(emitter, "__rt_str_persist");                          // rax/rdx = owned heap copy, non-null even at length zero
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning the measured string result
    emitter.instruction("ret");                                                 // return to the caller with the owned string ptr/len in the x86_64 result regs
}

/// Emits `__rt_getenv_all`, which returns a fresh associative string map of the process environment.
pub fn emit_getenv_all(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_getenv_all_aarch64(emitter),
        Arch::X86_64 => emit_getenv_all_linux_x86_64(emitter),
    }
}

/// Emits the AArch64 environment enumeration loop for macOS and Linux.
fn emit_getenv_all_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: getenv_all ---");
    emitter.label_global("__rt_getenv_all");
    emitter.instruction("sub sp, sp, #96");                                     // reserve cursor, hash, entry, key/value, and normalized-key spill slots
    emitter.instruction("stp x29, x30, [sp, #80]");                             // preserve the caller frame pointer and return address
    emitter.instruction("add x29, sp, #80");                                    // establish a stable frame for nested runtime calls

    match emitter.platform {
        Platform::MacOS => {
            emitter.bl_c("_NSGetEnviron");                                    // fetch the Darwin pointer-to-environment-list accessor result
            emitter.instruction("ldr x9, [x0]");                                // dereference char*** into the current char** environment list
        }
        Platform::Linux => {
            abi::emit_extern_symbol_address(emitter, "x9", "environ");         // resolve libc's char** environment variable through the GOT
            emitter.instruction("ldr x9, [x9]");                                // dereference the global into the current environment list
        }
        Platform::Windows => unreachable!("Windows target is not yet supported"),
    }
    emitter.instruction("str x9, [sp]");                                        // save the current char** cursor
    emitter.instruction("mov x0, #16");                                         // start with a small table that can grow as entries are inserted
    emitter.instruction(&format!("mov x1, #{}", STR_TAG));                      // declare string values in the hash header
    emitter.instruction("bl __rt_hash_new");                                    // allocate the fresh result hash
    emitter.instruction("str x0, [sp, #8]");                                    // preserve the current result hash across the enumeration loop

    emitter.label("__rt_getenv_all_next");
    emitter.instruction("ldr x9, [sp]");                                        // reload the current char** cursor
    emitter.instruction("ldr x10, [x9]");                                       // load the next KEY=VALUE entry pointer
    emitter.instruction("cbz x10, __rt_getenv_all_done");                       // a null entry terminates the environment list
    emitter.instruction("add x9, x9, #8");                                      // advance to the following char* slot
    emitter.instruction("str x9, [sp]");                                        // save the advanced cursor before nested calls
    emitter.instruction("str x10, [sp, #16]");                                  // preserve the entry pointer as the raw key start
    emitter.instruction("mov x11, #0");                                         // initialize the key-length scan

    emitter.label("__rt_getenv_all_key_scan");
    emitter.instruction("ldrb w12, [x10, x11]");                                // load the next byte while searching for the first equals sign
    emitter.instruction("cbz w12, __rt_getenv_all_next");                       // skip malformed entries that contain no separator
    emitter.instruction("cmp w12, #61");                                        // is this byte the KEY=VALUE separator?
    emitter.instruction("b.eq __rt_getenv_all_key_done");                       // stop at the first separator so later equals signs stay in the value
    emitter.instruction("add x11, x11, #1");                                    // extend the key by one byte
    emitter.instruction("b __rt_getenv_all_key_scan");                          // continue scanning the current entry

    emitter.label("__rt_getenv_all_key_done");
    emitter.instruction("str x11, [sp, #24]");                                  // save the raw key length
    emitter.instruction("add x12, x10, x11");                                   // advance from the entry start to the separator
    emitter.instruction("add x12, x12, #1");                                    // skip the separator to the value start
    emitter.instruction("str x12, [sp, #32]");                                  // preserve the raw value pointer
    emitter.instruction("mov x13, #0");                                         // initialize the value-length scan

    emitter.label("__rt_getenv_all_value_scan");
    emitter.instruction("ldrb w14, [x12, x13]");                                // load the next value byte
    emitter.instruction("cbz w14, __rt_getenv_all_value_done");                 // stop at the C-string terminator
    emitter.instruction("add x13, x13, #1");                                    // extend the value by one byte
    emitter.instruction("b __rt_getenv_all_value_scan");                        // continue measuring the value

    emitter.label("__rt_getenv_all_value_done");
    emitter.instruction("str x13, [sp, #40]");                                  // save the raw value length
    emitter.instruction("ldr x1, [sp, #16]");                                   // pass the raw key pointer for PHP key normalization
    emitter.instruction("ldr x2, [sp, #24]");                                   // pass the raw key length
    emitter.instruction("bl __rt_hash_normalize_key");                          // normalize numeric-looking string keys consistently with PHP arrays
    emitter.instruction("str x1, [sp, #48]");                                   // preserve the normalized key pointer
    emitter.instruction("str x2, [sp, #56]");                                   // preserve the normalized key length
    emitter.instruction("ldr x1, [sp, #32]");                                   // pass the raw environment value pointer
    emitter.instruction("ldr x2, [sp, #40]");                                   // pass the raw environment value length
    emitter.instruction("bl __rt_str_persist");                                 // copy the value out of mutable process-owned storage
    emitter.instruction("mov x3, x1");                                          // pass the owned value pointer to the hash setter
    emitter.instruction("mov x4, x2");                                          // pass the owned value length to the hash setter
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the current result hash
    emitter.instruction("ldr x1, [sp, #48]");                                   // reload the normalized key pointer
    emitter.instruction("ldr x2, [sp, #56]");                                   // reload the normalized key length
    emitter.instruction(&format!("mov x5, #{}", STR_TAG));                      // mark the inserted value as a string
    emitter.instruction("bl __rt_hash_set");                                    // insert the copied entry, growing the hash if required
    emitter.instruction("str x0, [sp, #8]");                                    // preserve the possibly grown result hash
    emitter.instruction("b __rt_getenv_all_next");                              // continue with the next process entry

    emitter.label("__rt_getenv_all_done");
    emitter.instruction("ldr x0, [sp, #8]");                                    // return the completed associative hash
    emitter.instruction("ldp x29, x30, [sp, #80]");                             // restore the caller frame pointer and return address
    emitter.instruction("add sp, sp, #96");                                     // release enumeration spill storage
    emitter.instruction("ret");                                                 // return the fresh hash pointer
}

/// Emits the System V x86_64 environment enumeration loop for Linux.
fn emit_getenv_all_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: getenv_all ---");
    emitter.label_global("__rt_getenv_all");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame for nested runtime calls
    emitter.instruction("sub rsp, 80");                                         // reserve cursor, hash, entry, key/value, and normalized-key spill slots
    abi::emit_extern_symbol_address(emitter, "rax", "environ");                 // resolve libc's char** environment variable through the GOT
    emitter.instruction("mov rax, QWORD PTR [rax]");                            // dereference the global into the current environment list
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the current char** cursor
    emitter.instruction("mov rdi, 16");                                         // start with a small table that can grow during insertion
    emitter.instruction(&format!("mov rsi, {}", STR_TAG));                      // declare string values in the hash header
    emitter.instruction("call __rt_hash_new");                                  // allocate the fresh result hash
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // preserve the current result hash across the loop

    emitter.label("__rt_getenv_all_next");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the current char** cursor
    emitter.instruction("mov r10, QWORD PTR [rax]");                            // load the next KEY=VALUE entry pointer
    emitter.instruction("test r10, r10");                                       // did the environment list reach its null terminator?
    emitter.instruction("jz __rt_getenv_all_done");                             // finish when no entries remain
    emitter.instruction("add rax, 8");                                          // advance to the following char* slot
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the advanced cursor before nested calls
    emitter.instruction("mov QWORD PTR [rbp - 24], r10");                       // preserve the entry pointer as the raw key start
    emitter.instruction("xor r11d, r11d");                                      // initialize the key-length scan

    emitter.label("__rt_getenv_all_key_scan");
    emitter.instruction("mov al, BYTE PTR [r10 + r11]");                        // load the next byte while searching for the first equals sign
    emitter.instruction("test al, al");                                         // did a malformed entry end before a separator?
    emitter.instruction("jz __rt_getenv_all_next");                             // skip entries that contain no separator
    emitter.instruction("cmp al, 61");                                          // is this byte the KEY=VALUE separator?
    emitter.instruction("je __rt_getenv_all_key_done");                         // stop at the first separator
    emitter.instruction("add r11, 1");                                          // extend the key by one byte
    emitter.instruction("jmp __rt_getenv_all_key_scan");                        // continue scanning the current entry

    emitter.label("__rt_getenv_all_key_done");
    emitter.instruction("mov QWORD PTR [rbp - 32], r11");                       // save the raw key length
    emitter.instruction("lea r10, [r10 + r11 + 1]");                            // compute the value start immediately after the separator
    emitter.instruction("mov QWORD PTR [rbp - 40], r10");                       // preserve the raw value pointer
    emitter.instruction("xor r11d, r11d");                                      // initialize the value-length scan

    emitter.label("__rt_getenv_all_value_scan");
    emitter.instruction("mov al, BYTE PTR [r10 + r11]");                        // load the next value byte
    emitter.instruction("test al, al");                                         // did the value reach its C-string terminator?
    emitter.instruction("jz __rt_getenv_all_value_done");                       // stop once the complete value length is known
    emitter.instruction("add r11, 1");                                          // extend the value by one byte
    emitter.instruction("jmp __rt_getenv_all_value_scan");                      // continue measuring the value

    emitter.label("__rt_getenv_all_value_done");
    emitter.instruction("mov QWORD PTR [rbp - 48], r11");                       // save the raw value length
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // pass the raw key pointer for PHP key normalization
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                       // pass the raw key length
    emitter.instruction("call __rt_hash_normalize_key");                        // normalize numeric-looking string keys consistently with PHP arrays
    emitter.instruction("mov QWORD PTR [rbp - 56], rax");                       // preserve the normalized key pointer
    emitter.instruction("mov QWORD PTR [rbp - 64], rdx");                       // preserve the normalized key length
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // pass the raw environment value pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 48]");                       // pass the raw environment value length
    emitter.instruction("call __rt_str_persist");                               // copy the value out of mutable process-owned storage
    emitter.instruction("mov rcx, rax");                                        // pass the owned value pointer to the hash setter
    emitter.instruction("mov r8, rdx");                                         // pass the owned value length to the hash setter
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // reload the current result hash
    emitter.instruction("mov rsi, QWORD PTR [rbp - 56]");                       // reload the normalized key pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 64]");                       // reload the normalized key length
    emitter.instruction(&format!("mov r9, {}", STR_TAG));                       // mark the inserted value as a string
    emitter.instruction("call __rt_hash_set");                                  // insert the copied entry, growing the hash if required
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // preserve the possibly grown result hash
    emitter.instruction("jmp __rt_getenv_all_next");                            // continue with the next process entry

    emitter.label("__rt_getenv_all_done");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // return the completed associative hash
    emitter.instruction("add rsp, 80");                                         // release enumeration spill storage
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the fresh hash pointer
}
