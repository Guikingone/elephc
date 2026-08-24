//! Purpose:
//! Emits the `__rt_chmod`, `__rt_cstr` runtime helper assembly for modify Linux x86 64.
//! Keeps PHP filesystem/resource behavior, libc calls, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//!
//! Key details:
//! - I/O helpers bridge PHP strings, resources, descriptors, and libc calls while returning runtime arrays or pointer/length strings.

use crate::codegen_support::runtime::data::{
    CHMOD_URL_WARNING_HEAD, CHMOD_URL_WARNING_MIDDLE, CHMOD_WARNING_HEAD,
    TOUCH_URL_WARNING_HEAD, TOUCH_URL_WARNING_MIDDLE, TOUCH_WARNING_HEAD,
    TOUCH_WARNING_MIDDLE,
};
use crate::codegen_support::{abi, emit::Emitter};

/// Emits x86_64 Linux runtime helpers for filesystem modify operations:
/// `__rt_chmod`, `__rt_chown`, `__rt_chown_user`, `__rt_chgrp_group`, `__rt_umask`,
/// `__rt_ftruncate`, `__rt_fsync`, `__rt_fflush`, `__rt_fdatasync`, `__rt_touch`.
///
/// Each helper converts a PHP string path to a C string, calls the corresponding
/// libc function, and returns a boolean predicate (1 on success, 0 on failure).
/// The `__rt_touch` helper additionally handles timestamp via `utimensat` using
/// platform-specific flags and OpenBSD-style atime/mtime masks passed via registers.
///
/// # Arguments
/// * `emitter` - The assembly emitter used to write x86_64 instructions.
///
/// # ABI details
/// * Path and string length arrive via `rdi`/`rsi` registers; `__rt_cstr` converts
///   to a C string pointer returned in `rax`.
/// * Scalar secondary arguments (mode, uid, gid, size) arrive via stack spill or
///   secondary registers (`rdx`, `rcx`).
/// * Boolean results are zero-extended to a full integer in `rax` before returning.
pub(super) fn emit_modify_linux_x86_64(emitter: &mut Emitter) {
    // -- chmod --
    emitter.blank();
    emitter.comment("--- runtime: chmod ---");
    emitter.label_global("__rt_chmod");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame
    emitter.instruction("sub rsp, 32");                                         // align stack, plus the path the warning names
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve mode (came in via the secondary string-argument register)
    emitter.instruction("call __rt_path_cstr");                                 // path → C string in rax
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // the path the URL shape names
    emitter.instruction("mov rdi, rax");                                        // first libc chmod arg = C path
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // second libc chmod arg = mode
    emitter.instruction("call chmod");                                          // libc chmod(path, mode)
    // See the AArch64 counterpart: php names no path for a direct call and BOTH a path and a
    // different wording when the argument was a `file://` URL.
    emitter.instruction("cmp eax, 0");                                          // did libc chmod() return success as a C int?
    emitter.instruction("je __rt_chmod_ok_x86");
    abi::emit_load_symbol_to_reg(emitter, "r10", "_rt_path_url", 0);
    emitter.instruction("test r10, r10");
    emitter.instruction("jnz __rt_chmod_url_warn_x86");
    super::path_op_warning::emit_libc_call_x86_64(
        emitter,
        "_warn_chmod_head",
        CHMOD_WARNING_HEAD.len(),
        None,
        "_warn_chmod_head",
        0,
    );
    emitter.instruction("jmp __rt_chmod_warned_x86");
    emitter.label("__rt_chmod_url_warn_x86");
    super::path_op_warning::emit_libc_call_x86_64(
        emitter,
        "_warn_chmod_url_head",
        CHMOD_URL_WARNING_HEAD.len(),
        Some("[rbp - 16]"),
        "_warn_chmod_url_mid",
        CHMOD_URL_WARNING_MIDDLE.len(),
    );
    emitter.label("__rt_chmod_warned_x86");
    emitter.instruction("xor eax, eax");                                        // php answers false for the failure it just named
    emitter.instruction("jmp __rt_chmod_done_x86");
    emitter.label("__rt_chmod_ok_x86");
    emitter.instruction("mov eax, 1");
    emitter.label("__rt_chmod_done_x86");
    emitter.instruction("add rsp, 32");                                         // release stack
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return predicate

    // -- chown --
    emitter.blank();
    emitter.comment("--- runtime: chown ---");
    emitter.label_global("__rt_chown");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame
    emitter.instruction("sub rsp, 32");                                         // align stack + spill uid/gid
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve uid
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // preserve gid
    emitter.instruction("call __rt_path_cstr");                                 // path → C string in rax
    emitter.instruction("mov rdi, rax");                                        // first libc chown arg = path
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // second arg = uid
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // third arg = gid
    emitter.instruction("call chown");                                          // libc chown(path, uid, gid)
    emitter.instruction("cmp eax, 0");                                          // did libc chown() return success as a C int?
    emitter.instruction("sete al");                                             // boolean byte
    emitter.instruction("movzx rax, al");                                       // widen
    emitter.instruction("add rsp, 32");                                         // release stack
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return predicate

    // -- lchown --
    emitter.blank();
    emitter.comment("--- runtime: lchown ---");
    emitter.label_global("__rt_lchown");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame
    emitter.instruction("sub rsp, 32");                                         // align stack + spill uid/gid
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve uid
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // preserve gid
    emitter.instruction("call __rt_path_cstr");                                 // path → C string in rax
    emitter.instruction("mov rdi, rax");                                        // first libc lchown arg = path
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // second arg = uid
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // third arg = gid
    emitter.instruction("call lchown");                                         // libc lchown(path, uid, gid) without following symlinks
    emitter.instruction("cmp eax, 0");                                          // did libc lchown() return success as a C int?
    emitter.instruction("sete al");                                             // boolean byte
    emitter.instruction("movzx rax, al");                                       // widen
    emitter.instruction("add rsp, 32");                                         // release stack
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return predicate

    // -- chown by user name --
    emitter.blank();
    emitter.comment("--- runtime: chown user name ---");
    emitter.label_global("__rt_chown_user");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame
    emitter.instruction("sub rsp, 32");                                         // align stack + spill user string and path pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // preserve user-name pointer
    emitter.instruction("mov QWORD PTR [rbp - 24], rsi");                       // preserve user-name length
    emitter.instruction("call __rt_path_cstr");                                 // path → C string in rax
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save C path pointer
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload user-name pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // reload user-name length
    emitter.instruction("call __rt_cstr2");                                     // user name → secondary C string in rax
    emitter.instruction("mov rdi, rax");                                        // first lookup arg = C user name
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // second lookup arg = user-name length
    emitter.instruction("call __rt_lookup_passwd_uid");                         // resolve uid from /etc/passwd without NSS
    emitter.instruction("cmp eax, -1");                                         // was the user name absent?
    emitter.instruction("je __rt_chown_user_fail_x86");                         // unknown user name → false
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // first chown arg = C path
    emitter.instruction("mov esi, eax");                                        // second chown arg = resolved uid
    emitter.instruction("mov rdx, -1");                                         // gid = -1 (leave group unchanged)
    emitter.instruction("call chown");                                          // libc chown(path, uid, -1)
    emitter.instruction("cmp eax, 0");                                          // did libc chown() return success as a C int?
    emitter.instruction("sete al");                                             // boolean byte
    emitter.instruction("movzx rax, al");                                       // widen
    emitter.instruction("jmp __rt_chown_user_done_x86");                        // skip failure return
    emitter.label("__rt_chown_user_fail_x86");
    emitter.instruction("xor eax, eax");                                        // unknown name returns false
    emitter.label("__rt_chown_user_done_x86");
    emitter.instruction("add rsp, 32");                                         // release stack
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return predicate

    // -- lchown by user name --
    emitter.blank();
    emitter.comment("--- runtime: lchown user name ---");
    emitter.label_global("__rt_lchown_user");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame
    emitter.instruction("sub rsp, 32");                                         // align stack + spill user string and path pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // preserve user-name pointer
    emitter.instruction("mov QWORD PTR [rbp - 24], rsi");                       // preserve user-name length
    emitter.instruction("call __rt_path_cstr");                                 // path → C string in rax
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save C path pointer
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload user-name pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // reload user-name length
    emitter.instruction("call __rt_cstr2");                                     // user name → secondary C string in rax
    emitter.instruction("mov rdi, rax");                                        // first lookup arg = C user name
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // second lookup arg = user-name length
    emitter.instruction("call __rt_lookup_passwd_uid");                         // resolve uid from /etc/passwd without NSS
    emitter.instruction("cmp eax, -1");                                         // was the user name absent?
    emitter.instruction("je __rt_lchown_user_fail_x86");                        // unknown user name → false
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // first lchown arg = C path
    emitter.instruction("mov esi, eax");                                        // second lchown arg = resolved uid
    emitter.instruction("mov rdx, -1");                                         // gid = -1 (leave group unchanged)
    emitter.instruction("call lchown");                                         // libc lchown(path, uid, -1) without following symlinks
    emitter.instruction("cmp eax, 0");                                          // did libc lchown() return success as a C int?
    emitter.instruction("sete al");                                             // boolean byte
    emitter.instruction("movzx rax, al");                                       // widen
    emitter.instruction("jmp __rt_lchown_user_done_x86");                       // skip failure return
    emitter.label("__rt_lchown_user_fail_x86");
    emitter.instruction("xor eax, eax");                                        // unknown name returns false
    emitter.label("__rt_lchown_user_done_x86");
    emitter.instruction("add rsp, 32");                                         // release stack
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return predicate

    // -- chgrp by group name --
    emitter.blank();
    emitter.comment("--- runtime: chgrp group name ---");
    emitter.label_global("__rt_chgrp_group");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame
    emitter.instruction("sub rsp, 32");                                         // align stack + spill group string and path pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // preserve group-name pointer
    emitter.instruction("mov QWORD PTR [rbp - 24], rsi");                       // preserve group-name length
    emitter.instruction("call __rt_path_cstr");                                 // path → C string in rax
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save C path pointer
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload group-name pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // reload group-name length
    emitter.instruction("call __rt_cstr2");                                     // group name → secondary C string in rax
    emitter.instruction("mov rdi, rax");                                        // first lookup arg = C group name
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // second lookup arg = group-name length
    emitter.instruction("call __rt_lookup_group_gid");                          // resolve gid from /etc/group without NSS
    emitter.instruction("cmp eax, -1");                                         // was the group name absent?
    emitter.instruction("je __rt_chgrp_group_fail_x86");                        // unknown group name → false
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // first chown arg = C path
    emitter.instruction("mov rsi, -1");                                         // uid = -1 (leave owner unchanged)
    emitter.instruction("mov edx, eax");                                        // third chown arg = resolved gid
    emitter.instruction("call chown");                                          // libc chown(path, -1, gid)
    emitter.instruction("cmp eax, 0");                                          // did libc chown() return success as a C int?
    emitter.instruction("sete al");                                             // boolean byte
    emitter.instruction("movzx rax, al");                                       // widen
    emitter.instruction("jmp __rt_chgrp_group_done_x86");                       // skip failure return
    emitter.label("__rt_chgrp_group_fail_x86");
    emitter.instruction("xor eax, eax");                                        // unknown name returns false
    emitter.label("__rt_chgrp_group_done_x86");
    emitter.instruction("add rsp, 32");                                         // release stack
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return predicate

    // -- lchgrp by group name --
    emitter.blank();
    emitter.comment("--- runtime: lchgrp group name ---");
    emitter.label_global("__rt_lchgrp_group");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame
    emitter.instruction("sub rsp, 32");                                         // align stack + spill group string and path pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // preserve group-name pointer
    emitter.instruction("mov QWORD PTR [rbp - 24], rsi");                       // preserve group-name length
    emitter.instruction("call __rt_path_cstr");                                 // path → C string in rax
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save C path pointer
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload group-name pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // reload group-name length
    emitter.instruction("call __rt_cstr2");                                     // group name → secondary C string in rax
    emitter.instruction("mov rdi, rax");                                        // first lookup arg = C group name
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // second lookup arg = group-name length
    emitter.instruction("call __rt_lookup_group_gid");                          // resolve gid from /etc/group without NSS
    emitter.instruction("cmp eax, -1");                                         // was the group name absent?
    emitter.instruction("je __rt_lchgrp_group_fail_x86");                       // unknown group name → false
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // first lchown arg = C path
    emitter.instruction("mov rsi, -1");                                         // uid = -1 (leave owner unchanged)
    emitter.instruction("mov edx, eax");                                        // third lchown arg = resolved gid
    emitter.instruction("call lchown");                                         // libc lchown(path, -1, gid) without following symlinks
    emitter.instruction("cmp eax, 0");                                          // did libc lchown() return success as a C int?
    emitter.instruction("sete al");                                             // boolean byte
    emitter.instruction("movzx rax, al");                                       // widen
    emitter.instruction("jmp __rt_lchgrp_group_done_x86");                      // skip failure return
    emitter.label("__rt_lchgrp_group_fail_x86");
    emitter.instruction("xor eax, eax");                                        // unknown name returns false
    emitter.label("__rt_lchgrp_group_done_x86");
    emitter.instruction("add rsp, 32");                                         // release stack
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return predicate

    // -- umask --
    emitter.blank();
    emitter.comment("--- runtime: umask ---");
    emitter.label_global("__rt_umask");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame
    emitter.instruction("mov rdi, rax");                                        // mask comes in via the int return register
    emitter.instruction("call umask");                                          // libc umask(mask) — returns previous mask
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return previous mask

    // -- ftruncate --
    emitter.blank();
    emitter.comment("--- runtime: ftruncate ---");
    emitter.label_global("__rt_ftruncate");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame
    emitter.instruction("mov rdi, rax");                                        // fd → SysV first arg (size already in rsi from the caller)
    emitter.instruction("call ftruncate");                                      // libc ftruncate(fd, size)
    emitter.instruction("cmp eax, 0");                                          // did libc ftruncate() return success as a C int?
    emitter.instruction("sete al");                                             // boolean byte
    emitter.instruction("movzx rax, al");                                       // widen
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return predicate

    // -- fsync / fflush --
    emitter.blank();
    emitter.comment("--- runtime: fsync ---");
    emitter.label_global("__rt_fsync");
    emitter.label_global("__rt_fflush");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame
    emitter.instruction("mov rdi, rax");                                        // fd
    emitter.instruction("call fsync");                                          // libc fsync(fd)
    emitter.instruction("cmp eax, 0");                                          // did libc fsync() return success as a C int?
    emitter.instruction("sete al");                                             // boolean byte
    emitter.instruction("movzx rax, al");                                       // widen
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return predicate

    // -- fdatasync --
    emitter.blank();
    emitter.comment("--- runtime: fdatasync ---");
    emitter.label_global("__rt_fdatasync");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame
    emitter.instruction("mov rdi, rax");                                        // fd
    if emitter.platform == crate::codegen_support::platform::Platform::Linux {
        emitter.instruction("call fdatasync");                                  // libc fdatasync(fd) on Linux
    } else {
        emitter.instruction("call fsync");                                      // Darwin fallback
    }
    emitter.instruction("cmp eax, 0");                                          // did libc sync helper return success as a C int?
    emitter.instruction("sete al");                                             // boolean byte
    emitter.instruction("movzx rax, al");                                       // widen
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return predicate

    // -- touch --
    emitter.blank();
    emitter.comment("--- runtime: touch ---");
    emitter.label_global("__rt_touch");
    // php locates a wrapper for every path; a bare one is the plain-files wrapper.
    super::fopen::emit_refuse_when_file_wrapper_disabled(emitter, super::fopen::DisabledWrapperAnswer::Predicate(0));
    let plat = emitter.platform;
    let open_flags = plat.o_wronly_creat();
    let utime_now = plat.utime_now_nsec();
    // Frame layout (rbp-relative):
    //   [rbp -  8] : C path pointer
    //   [rbp - 16] : mtime arg
    //   [rbp - 24] : atime arg
    //   [rbp - 32] : current-time mask
    //   [rbp - 64] : timespec[0] (tv_sec=[rbp-64], tv_nsec=[rbp-56])
    //   [rbp - 48] : timespec[1] (tv_sec=[rbp-48], tv_nsec=[rbp-40])
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame
    emitter.instruction("sub rsp, 80");                                         // reserve aligned frame
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // save mtime arg
    emitter.instruction("mov QWORD PTR [rbp - 24], rsi");                       // save atime arg
    emitter.instruction("mov QWORD PTR [rbp - 32], rcx");                       // save current-time mask
    emitter.instruction("call __rt_path_cstr");                                 // path → C string in rax
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save C path pointer

    emitter.instruction("mov rdi, rax");                                        // first arg = path
    emitter.instruction(&format!("mov rsi, 0x{:X}", open_flags));               // open flags
    emitter.instruction("mov rdx, 0x1B6");                                      // mode 0666 before umask
    emitter.instruction("call open");                                           // libc open()
    emitter.instruction("cmp eax, 0");                                          // did libc open() return a negative C int?
    emitter.instruction("jl __rt_touch_set_times_x86");                         // skip close on failure
    emitter.instruction("cdqe");                                                // normalize the successful C int fd before closing it
    emitter.instruction("mov rdi, rax");                                        // fd
    emitter.instruction("call close");                                          // libc close(fd)

    emitter.label("__rt_touch_set_times_x86");
    // atime
    emitter.instruction("mov r8, QWORD PTR [rbp - 32]");                        // load current-time mask
    emitter.instruction("test r8, 1");                                          // use current time for atime?
    emitter.instruction("jnz __rt_touch_atime_now_x86");                        // current atime path
    emitter.instruction("mov r8, QWORD PTR [rbp - 24]");                        // load explicit atime seconds
    emitter.instruction("mov QWORD PTR [rbp - 64], r8");                        // tv_sec = atime
    emitter.instruction("mov QWORD PTR [rbp - 56], 0");                         // tv_nsec = 0
    emitter.instruction("jmp __rt_touch_handle_mtime_x86");                     // continue with mtime selection
    emitter.label("__rt_touch_atime_now_x86");
    emitter.instruction("mov QWORD PTR [rbp - 64], 0");                         // tv_sec = 0
    emitter.instruction(&format!("mov QWORD PTR [rbp - 56], {}", utime_now));   // tv_nsec = platform UTIME_NOW sentinel

    emitter.label("__rt_touch_handle_mtime_x86");
    emitter.instruction("mov r8, QWORD PTR [rbp - 32]");                        // reload current-time mask
    emitter.instruction("test r8, 2");                                          // use current time for mtime?
    emitter.instruction("jnz __rt_touch_mtime_now_x86");                        // current mtime path
    emitter.instruction("mov r8, QWORD PTR [rbp - 16]");                        // load explicit mtime seconds
    emitter.instruction("mov QWORD PTR [rbp - 48], r8");                        // tv_sec = mtime
    emitter.instruction("mov QWORD PTR [rbp - 40], 0");                         // tv_nsec = 0
    emitter.instruction("jmp __rt_touch_call_utimensat_x86");                   // call utimensat with prepared timestamps
    emitter.label("__rt_touch_mtime_now_x86");
    emitter.instruction("mov QWORD PTR [rbp - 48], 0");                         // tv_sec = 0
    emitter.instruction(&format!("mov QWORD PTR [rbp - 40], {}", utime_now));   // tv_nsec = platform UTIME_NOW sentinel

    emitter.label("__rt_touch_call_utimensat_x86");
    emitter.instruction(&format!("mov rdi, {}", plat.at_fdcwd()));              // AT_FDCWD (platform-dependent: -2 Darwin, -100 Linux)
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // C path pointer
    emitter.instruction("lea rdx, [rbp - 64]");                                 // pointer to timespec[0]
    emitter.instruction("mov rcx, 0");                                          // flags = 0
    emitter.instruction("call utimensat");                                      // libc utimensat()
    // See the AArch64 counterpart: the stamp is what decides, and a URL takes the wrapper hook's
    // shape, which names the path in the parentheses AND again in the sentence.
    emitter.instruction("cmp eax, 0");                                          // did libc utimensat() return success as a C int?
    emitter.instruction("je __rt_touch_ok_x86");
    abi::emit_load_symbol_to_reg(emitter, "r10", "_rt_path_url", 0);
    emitter.instruction("test r10, r10");
    emitter.instruction("jnz __rt_touch_url_warn_x86");
    super::path_op_warning::emit_libc_call_x86_64(
        emitter,
        "_warn_touch_head",
        TOUCH_WARNING_HEAD.len(),
        Some("[rbp - 8]"),
        "_warn_touch_mid",
        TOUCH_WARNING_MIDDLE.len(),
    );
    emitter.instruction("jmp __rt_touch_warned_x86");
    emitter.label("__rt_touch_url_warn_x86");
    abi::emit_symbol_address(emitter, "rdi", "_warn_touch_url_head");
    emitter.instruction(&format!("mov rsi, {}", TOUCH_URL_WARNING_HEAD.len()));
    emitter.instruction("mov rdx, QWORD PTR [rbp - 8]");
    abi::emit_symbol_address(emitter, "rcx", "_warn_touch_url_mid");
    emitter.instruction(&format!("mov r8, {}", TOUCH_URL_WARNING_MIDDLE.len()));
    emitter.instruction("call __rt_path_op_fragment");                          // head, path, the sentence's opening
    super::path_op_warning::emit_libc_call_x86_64(
        emitter,
        "_warn_touch_url_head",
        0,
        Some("[rbp - 8]"),
        "_warn_touch_mid",
        TOUCH_WARNING_MIDDLE.len(),
    );
    emitter.label("__rt_touch_warned_x86");
    emitter.instruction("xor eax, eax");                                        // php answers false for the failure it just named
    emitter.instruction("jmp __rt_touch_done_x86");
    emitter.label("__rt_touch_ok_x86");
    emitter.instruction("mov eax, 1");
    emitter.label("__rt_touch_done_x86");
    emitter.instruction("add rsp, 80");                                         // release frame
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return predicate
}
