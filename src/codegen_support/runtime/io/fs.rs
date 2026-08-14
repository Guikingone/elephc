//! Purpose:
//! Emits the `__rt_unlink`, `__rt_cstr` runtime helper assembly for fs.
//! Keeps PHP filesystem/resource behavior, libc calls, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//!
//! Key details:
//! - I/O helpers bridge PHP strings, resources, descriptors, and libc calls while returning runtime arrays or pointer/length strings.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits all filesystem runtime helpers: `__rt_unlink`, `__rt_mkdir`, `__rt_rmdir`,
/// `__rt_chdir`, `__rt_rename`, and `__rt_copy`.
///
/// Dispatches to `emit_fs_linux_x86_64` on x86_64 Linux; emits ARM64 syscall-based
/// helpers on all other targets. Each helper takes PHP string path arguments (x1=ptr,
/// x2=len) and returns x0=1 on success, 0 on failure.
pub fn emit_fs(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_fs_linux_x86_64(emitter);
        return;
    }

    // ================================================================
    // __rt_unlink: delete a file
    // Input:  x1/x2=path, x3=mode, x4=recursive
    // Output: x0=1 on success, 0 on failure
    // ================================================================
    emitter.blank();
    emitter.comment("--- runtime: unlink ---");
    emitter.label_global("__rt_unlink");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #16");                                     // allocate 16 bytes on the stack
    emitter.instruction("stp x29, x30, [sp]");                                  // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish new frame pointer

    // -- null-terminate path and call unlink --
    emitter.instruction("bl __rt_cstr");                                        // convert path to C string, x0=cstr
    emitter.syscall(10);

    // -- return success/failure --
    emitter.instruction("cmp x0, #0");                                          // check syscall result
    emitter.instruction("cset x0, eq");                                         // x0 = 1 if unlink succeeded

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp]");                                  // restore frame pointer and return address
    emitter.instruction("add sp, sp, #16");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller

    // ================================================================
    // __rt_mkdir: create a directory
    // Input:  x1/x2=path
    // Output: x0=1 on success, 0 on failure
    // ================================================================
    emitter.blank();
    emitter.comment("--- runtime: mkdir ---");
    emitter.label_global("__rt_mkdir");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #48");                                     // reserve mode, recursive flag, C path, scan pointer, and the saved frame pair
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish new frame pointer
    emitter.instruction("str x3, [sp, #0]");                                    // preserve requested permissions across C-string conversion and mkdir calls
    emitter.instruction("str x4, [sp, #8]");                                    // preserve whether parent directories should be created

    // -- null-terminate path and create missing parents when requested --
    emitter.instruction("bl __rt_cstr");                                        // convert path to C string, x0=cstr
    emitter.instruction("str x0, [sp, #16]");                                   // preserve the full mutable C path for the final mkdir call
    emitter.instruction("ldr x9, [sp, #8]");                                    // load the recursive flag
    emitter.instruction("cbz x9, __rt_mkdir_final");                            // non-recursive calls create only the requested directory
    emitter.instruction("mov x10, x0");                                         // scan the mutable C path for parent separators
    emitter.label("__rt_mkdir_recursive_scan");
    emitter.instruction("ldrb w11, [x10]");                                     // read the current path byte
    emitter.instruction("cbz w11, __rt_mkdir_final");                           // the terminator marks the final directory component
    emitter.instruction("cmp w11, #47");                                        // compare the byte with '/'
    emitter.instruction("b.ne __rt_mkdir_recursive_next");                      // ordinary component bytes need no action
    emitter.instruction("ldr x12, [sp, #16]");                                  // reload the full path start for leading-slash detection
    emitter.instruction("cmp x10, x12");                                        // compare the separator address with the first byte
    emitter.instruction("b.eq __rt_mkdir_recursive_next");                      // never try to create an empty root prefix
    emitter.instruction("ldrb w12, [x10, #1]");                                 // inspect the byte following the separator
    emitter.instruction("cbz w12, __rt_mkdir_recursive_next");                  // leave a trailing slash for the final mkdir call
    emitter.instruction("strb wzr, [x10]");                                     // temporarily terminate the current parent prefix
    emitter.instruction("str x10, [sp, #24]");                                  // preserve the scan pointer across the mkdir syscall
    emitter.instruction("ldr x0, [sp, #16]");                                   // pass the parent prefix C string
    emitter.instruction("ldr x1, [sp, #0]");                                    // pass requested permissions for every created parent
    emitter.syscall(136);
    emitter.instruction("ldr x10, [sp, #24]");                                  // restore the separator address after the syscall
    emitter.instruction("mov w11, #47");                                        // materialize '/' for path restoration
    emitter.instruction("strb w11, [x10]");                                     // restore the separator before scanning the next component
    emitter.label("__rt_mkdir_recursive_next");
    emitter.instruction("add x10, x10, #1");                                    // advance to the next path byte
    emitter.instruction("b __rt_mkdir_recursive_scan");                         // continue until the C-string terminator
    emitter.label("__rt_mkdir_final");
    emitter.instruction("ldr x0, [sp, #16]");                                   // pass the complete C path for the observable final mkdir call
    emitter.instruction("ldr x1, [sp, #0]");                                    // pass requested permissions and let the process umask apply
    emitter.syscall(136);

    // -- return success/failure --
    emitter.instruction("cmp x0, #0");                                          // check syscall result
    emitter.instruction("cset x0, eq");                                         // x0 = 1 if mkdir succeeded

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release mkdir scratch storage
    emitter.instruction("ret");                                                 // return to caller

    // ================================================================
    // __rt_rmdir: remove a directory
    // Input:  x1/x2=path
    // Output: x0=1 on success, 0 on failure
    // ================================================================
    emitter.blank();
    emitter.comment("--- runtime: rmdir ---");
    emitter.label_global("__rt_rmdir");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #16");                                     // allocate 16 bytes on the stack
    emitter.instruction("stp x29, x30, [sp]");                                  // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish new frame pointer

    // -- null-terminate path and call rmdir --
    emitter.instruction("bl __rt_cstr");                                        // convert path to C string, x0=cstr
    emitter.syscall(137);

    // -- return success/failure --
    emitter.instruction("cmp x0, #0");                                          // check syscall result
    emitter.instruction("cset x0, eq");                                         // x0 = 1 if rmdir succeeded

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp]");                                  // restore frame pointer and return address
    emitter.instruction("add sp, sp, #16");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller

    // ================================================================
    // __rt_chdir: change working directory
    // Input:  x1/x2=path
    // Output: x0=1 on success, 0 on failure
    // ================================================================
    emitter.blank();
    emitter.comment("--- runtime: chdir ---");
    emitter.label_global("__rt_chdir");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #16");                                     // allocate 16 bytes on the stack
    emitter.instruction("stp x29, x30, [sp]");                                  // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish new frame pointer

    // -- null-terminate path and call chdir --
    emitter.instruction("bl __rt_cstr");                                        // convert path to C string, x0=cstr
    emitter.syscall(12);

    // -- return success/failure --
    emitter.instruction("cmp x0, #0");                                          // check syscall result
    emitter.instruction("cset x0, eq");                                         // x0 = 1 if chdir succeeded

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp]");                                  // restore frame pointer and return address
    emitter.instruction("add sp, sp, #16");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller

    // ================================================================
    // __rt_rename: rename a file or directory
    // Input:  x1/x2=from path, x3/x4=to path
    // Output: x0=1 on success, 0 on failure
    // ================================================================
    emitter.blank();
    emitter.comment("--- runtime: rename ---");
    emitter.label_global("__rt_rename");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #48");                                     // allocate 48 bytes on the stack
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish new frame pointer

    // -- save destination path before clobbering registers --
    emitter.instruction("stp x3, x4, [sp, #16]");                               // save 'to' path ptr and len on stack

    // -- null-terminate source path using primary buffer --
    emitter.instruction("bl __rt_cstr");                                        // convert 'from' to C string in _cstr_buf
    emitter.instruction("str x0, [sp, #0]");                                    // save source cstr pointer

    // -- null-terminate destination path using secondary buffer --
    emitter.instruction("ldp x1, x2, [sp, #16]");                               // reload 'to' path ptr and len
    emitter.instruction("bl __rt_cstr2");                                       // convert 'to' to C string in _cstr_buf2
    emitter.instruction("str x0, [sp, #8]");                                    // save destination cstr pointer

    // -- call rename syscall --
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload source cstr path
    emitter.instruction("ldr x1, [sp, #8]");                                    // reload destination cstr path
    emitter.syscall(128);

    // -- return success/failure --
    emitter.instruction("cmp x0, #0");                                          // check syscall result
    emitter.instruction("cset x0, eq");                                         // x0 = 1 if rename succeeded

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller

    // ================================================================
    // __rt_copy: copy a file
    // Input:  x1/x2=from path, x3/x4=to path
    // Output: x0=1 on success, 0 on failure
    // ================================================================
    emitter.blank();
    emitter.comment("--- runtime: copy ---");
    emitter.label_global("__rt_copy");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #48");                                     // allocate 48 bytes on the stack
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish new frame pointer

    // -- save destination path for after reading source --
    emitter.instruction("stp x3, x4, [sp, #16]");                               // save 'to' path ptr and len on stack

    // -- read source file contents --
    emitter.instruction("bl __rt_file_get_contents");                           // read source, x1=data ptr, x2=data len

    // -- write contents to destination file --
    emitter.instruction("mov x3, x1");                                          // move data ptr to x3 (data arg)
    emitter.instruction("mov x4, x2");                                          // move data len to x4 (data arg)
    emitter.instruction("ldp x1, x2, [sp, #16]");                               // reload destination path ptr and len
    emitter.instruction("bl __rt_file_put_contents");                           // write data to dest file, x0=bytes written

    // -- return 1 if bytes were written --
    emitter.instruction("cmp x0, #0");                                          // check if any bytes were written
    emitter.instruction("cset x0, gt");                                         // x0 = 1 if bytes written > 0

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits x86_64 Linux variants of all filesystem helpers using libc calls.
/// Uses a stack-based frame (rbp/rsp convention) instead of the ARM64 link-register frame.
/// Return value convention matches `emit_fs`: x0=1 on success, 0 on failure.
fn emit_fs_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: unlink ---");
    emitter.label_global("__rt_unlink");
    emit_single_path_libc_bool_helper(emitter, "unlink", None);

    emitter.blank();
    emitter.comment("--- runtime: mkdir ---");
    emitter.label_global("__rt_mkdir");
    emit_mkdir_libc_bool_helper(emitter);

    emitter.blank();
    emitter.comment("--- runtime: rmdir ---");
    emitter.label_global("__rt_rmdir");
    emit_single_path_libc_bool_helper(emitter, "rmdir", None);

    emitter.blank();
    emitter.comment("--- runtime: chdir ---");
    emitter.label_global("__rt_chdir");
    emit_single_path_libc_bool_helper(emitter, "chdir", None);

    emitter.blank();
    emitter.comment("--- runtime: rename ---");
    emitter.label_global("__rt_rename");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer while rename uses temporary path slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the source and destination path temporaries
    emitter.instruction("sub rsp, 32");                                         // reserve aligned stack space for the saved destination and source C-string pointers
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the destination elephc path pointer while converting the source path
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the destination elephc path length while converting the source path
    emitter.instruction("call __rt_cstr");                                      // convert the source elephc path in rax/rdx into a null-terminated C string
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the source C-string pointer for the later libc rename() call
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the destination elephc path pointer before converting it to a C string
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // reload the destination elephc path length before converting it to a C string
    emitter.instruction("call __rt_cstr2");                                     // convert the destination elephc path into the secondary null-terminated C string buffer
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the destination C-string pointer for the later libc rename() call
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // pass the source C-string pointer as the first libc rename() argument
    emitter.instruction("mov rsi, QWORD PTR [rbp - 32]");                       // pass the destination C-string pointer as the second libc rename() argument
    emitter.instruction("call rename");                                         // rename or move the file-system path through libc rename()
    emitter.instruction("cmp eax, 0");                                          // a successful libc rename() call returns zero as a C int
    emitter.instruction("sete al");                                             // convert the rename() success flag into a boolean byte
    emitter.instruction("movzx rax, al");                                       // widen the boolean byte into the canonical integer result register
    emitter.instruction("add rsp, 32");                                         // release the aligned stack locals used by rename()
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the rename() success predicate to the caller

    emitter.blank();
    emitter.comment("--- runtime: copy ---");
    emitter.label_global("__rt_copy");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer while copy() uses path and payload spill slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the saved destination path and copied file payload
    emitter.instruction("sub rsp, 32");                                         // reserve aligned stack space for the destination path pair and copied payload pair
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the destination elephc path pointer while the source file is read into owned storage
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the destination elephc path length while the source file is read into owned storage
    emitter.instruction("call __rt_file_get_contents");                         // read the source file into an owned elephc string before writing it to the destination path
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // preserve the copied file payload pointer across the destination-path reload and write helper call
    emitter.instruction("mov QWORD PTR [rbp - 32], rdx");                       // preserve the copied file payload length across the destination-path reload and write helper call
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the destination elephc path pointer into the primary x86_64 string argument register
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // reload the destination elephc path length into the primary x86_64 string length register
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // pass the copied file payload pointer as the data pointer argument to file_put_contents()
    emitter.instruction("mov rsi, QWORD PTR [rbp - 32]");                       // pass the copied file payload length as the data length argument to file_put_contents()
    emitter.instruction("call __rt_file_put_contents");                         // write the copied file payload into the destination path through the shared file_put_contents() helper
    emitter.instruction("cmp rax, 0");                                          // treat zero-byte writes as success so empty files can still be copied correctly
    emitter.instruction("setge al");                                            // convert the signed write result into a boolean success byte where any non-negative byte count is success
    emitter.instruction("movzx rax, al");                                       // widen the boolean success byte into the canonical integer result register
    emitter.instruction("add rsp, 32");                                         // release the aligned stack locals used by copy()
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning the copy() success predicate
    emitter.instruction("ret");                                                 // return the copy() success predicate to the caller

}

/// Emits a leaf helper for single-path libc filesystem functions on x86_64.
///
/// Takes an optional setup instruction string inserted before the libc call to populate
/// extra arguments (e.g., mode for `mkdir`). The C path is passed via `__rt_cstr`
/// output in `rax`; the libc result is compared against 0 and returned as 1 (success) or
/// 0 (failure) in `rax`.
fn emit_single_path_libc_bool_helper(emitter: &mut Emitter, symbol: &str, extra_setup: Option<&str>) {
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer while the helper makes libc calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the call-aligned helper body
    emitter.instruction("call __rt_cstr");                                      // convert the elephc path in rax/rdx into a null-terminated C string in rax
    emitter.instruction("mov rdi, rax");                                        // pass the C path pointer as the first libc argument
    if let Some(setup) = extra_setup {
        emitter.instruction(setup);                                             // populate any additional libc arguments required by this helper
    }
    emitter.instruction(&format!("call {}", symbol));                           // invoke the matching libc file-system helper on Linux x86_64
    emitter.instruction("cmp eax, 0");                                          // libc path helpers return zero as a C int on success
    emitter.instruction("sete al");                                             // convert the success code into a boolean byte
    emitter.instruction("movzx rax, al");                                       // widen the boolean byte into the canonical integer result register
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer after the libc helper returns
    emitter.instruction("ret");                                                 // return the file-system success predicate to the caller
}

/// Emits x86_64 directory creation with optional parent-prefix creation.
///
/// The helper receives the path in `rax`/`rdx`, permissions in `rdi`, and the
/// recursive flag in `rsi`. Intermediate parent failures are deferred to the
/// final full-path `mkdir`, whose result is the observable PHP boolean.
fn emit_mkdir_libc_bool_helper(emitter: &mut Emitter) {
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer while mkdir scans a mutable C path
    emitter.instruction("mov rbp, rsp");                                        // establish a stable base for permissions, flags, path, and scan-pointer slots
    emitter.instruction("sub rsp, 32");                                         // reserve four aligned scratch words
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve requested permissions across C-string conversion and mkdir calls
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // preserve whether parent directories should be created
    emitter.instruction("call __rt_cstr");                                      // convert the elephc path in rax/rdx into a mutable C string
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // preserve the full C path for the final mkdir call
    emitter.instruction("cmp QWORD PTR [rbp - 16], 0");                         // test the recursive flag
    emitter.instruction("je __rt_mkdir_x86_final");                             // non-recursive calls create only the requested directory
    emitter.instruction("mov rcx, rax");                                        // scan the mutable C path for parent separators
    emitter.label("__rt_mkdir_x86_recursive_scan");
    emitter.instruction("movzx edx, BYTE PTR [rcx]");                           // read the current path byte
    emitter.instruction("test dl, dl");                                         // test for the C-string terminator
    emitter.instruction("jz __rt_mkdir_x86_final");                             // the terminator marks the final directory component
    emitter.instruction("cmp dl, 47");                                          // compare the byte with '/'
    emitter.instruction("jne __rt_mkdir_x86_recursive_next");                   // ordinary component bytes need no action
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 24]");                       // compare the separator address with the first byte
    emitter.instruction("je __rt_mkdir_x86_recursive_next");                    // never try to create an empty root prefix
    emitter.instruction("cmp BYTE PTR [rcx + 1], 0");                           // inspect the byte following the separator
    emitter.instruction("je __rt_mkdir_x86_recursive_next");                    // leave a trailing slash for the final mkdir call
    emitter.instruction("mov BYTE PTR [rcx], 0");                               // temporarily terminate the current parent prefix
    emitter.instruction("mov QWORD PTR [rbp - 32], rcx");                       // preserve the scan pointer across the libc call
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // pass the parent prefix C string
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // pass requested permissions for every created parent
    emitter.instruction("call mkdir");                                          // create the current parent and defer any failure to the full-path call
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");                       // restore the separator address after libc
    emitter.instruction("mov BYTE PTR [rcx], 47");                              // restore '/' before scanning the next component
    emitter.label("__rt_mkdir_x86_recursive_next");
    emitter.instruction("inc rcx");                                             // advance to the next path byte
    emitter.instruction("jmp __rt_mkdir_x86_recursive_scan");                   // continue until the C-string terminator
    emitter.label("__rt_mkdir_x86_final");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // pass the complete C path for the observable final mkdir call
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // pass requested permissions and let the process umask apply
    emitter.instruction("call mkdir");                                          // create the requested directory through libc
    emitter.instruction("cmp eax, 0");                                          // libc mkdir returns zero on success
    emitter.instruction("sete al");                                             // convert the success code into a boolean byte
    emitter.instruction("movzx rax, al");                                       // widen the boolean into the canonical result register
    emitter.instruction("add rsp, 32");                                         // release mkdir scratch storage
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the directory-creation success predicate
}
