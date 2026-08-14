//! Purpose:
//! Emits OpenSSL cipher metadata helpers backed by the elephc-crypto bridge.
//!
//! Called from:
//! - `runtime::emitters::emit_runtime()` through the string-runtime module.
//!
//! Key details:
//! - Function pointers are published only at OpenSSL call sites.
//! - The method bridge returns trailing-NUL packed names, which are copied into owned array elements.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits IV-length lookup and supported-cipher-list runtime helpers.
pub fn emit_openssl_methods(emitter: &mut Emitter) {
    emit_openssl_cipher_iv_length(emitter);
    emit_openssl_get_cipher_methods(emitter);
}

/// Emits `__rt_openssl_cipher_iv_length`, returning a length or negative bridge status.
fn emit_openssl_cipher_iv_length(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: openssl_cipher_iv_length ---");
    emitter.label_global("__rt_openssl_cipher_iv_length");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("stp x29, x30, [sp, #-16]!");                   // preserve the helper frame across the indirect bridge call
            emitter.instruction("mov x29, sp");                                 // establish a conventional AArch64 frame
            emitter.instruction("mov x0, x1");                                  // C arg0 = cipher-name pointer
            emitter.instruction("mov x1, x2");                                  // C arg1 = cipher-name length
            abi::emit_symbol_address(emitter, "x9", "_elephc_crypto_cipher_iv_length_fn");
            emitter.instruction("ldr x9, [x9]");                                // load the published IV-length bridge entry
            emitter.instruction("cbz x9, __rt_openssl_iv_length_missing");      // a missing bridge behaves like an unknown cipher
            abi::emit_call_reg(emitter, "x9");
            emitter.instruction("b __rt_openssl_iv_length_done");               // retain the bridge result in x0
            emitter.label("__rt_openssl_iv_length_missing");
            emitter.instruction("mov x0, #-1");                                 // stable unknown-cipher status
            emitter.label("__rt_openssl_iv_length_done");
            emitter.instruction("ldp x29, x30, [sp], #16");                     // restore the caller frame
            emitter.instruction("ret");                                         // return length/status in x0
        }
        Arch::X86_64 => {
            emitter.instruction("push rbp");                                    // preserve the caller frame pointer
            emitter.instruction("mov rbp, rsp");                                // establish an aligned SysV frame
            emitter.instruction("mov rdi, rax");                                // C arg0 = cipher-name pointer
            emitter.instruction("mov rsi, rdx");                                // C arg1 = cipher-name length
            abi::emit_load_symbol_to_reg(
                emitter,
                "r11",
                "_elephc_crypto_cipher_iv_length_fn",
                0,
            );
            emitter.instruction("test r11, r11");                               // check whether the bridge entry was published
            emitter.instruction("jz __rt_openssl_iv_length_missing_linux_x86_64");
            abi::emit_call_reg(emitter, "r11");
            emitter.instruction("jmp __rt_openssl_iv_length_done_linux_x86_64");
            emitter.label("__rt_openssl_iv_length_missing_linux_x86_64");
            emitter.instruction("mov rax, -1");                                 // stable unknown-cipher status
            emitter.label("__rt_openssl_iv_length_done_linux_x86_64");
            emitter.instruction("pop rbp");                                     // restore the caller frame pointer
            emitter.instruction("ret");                                         // return length/status in rax
        }
    }
}

/// Emits `__rt_openssl_get_cipher_methods`, decoding the bridge's packed inventory.
fn emit_openssl_get_cipher_methods(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_openssl_get_cipher_methods_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: openssl_get_cipher_methods ---");
    emitter.label_global("__rt_openssl_get_cipher_methods");
    emitter.instruction("sub sp, sp, #560");                                    // reserve packed-name buffer, state, and saved frame
    emitter.instruction("add x9, sp, #544");                                    // address saved-frame area beyond pair-immediate range
    emitter.instruction("stp x29, x30, [x9]");                                  // preserve frame pointer and return address
    emitter.instruction("mov x29, x9");                                         // establish the helper frame
    emitter.instruction("add x1, sp, #0");                                      // C arg1 = packed-name output buffer
    emitter.instruction("mov x2, #512");                                        // C arg2 = packed-name buffer capacity
    emitter.instruction("add x3, sp, #512");                                    // C arg3 = packed output byte length
    abi::emit_symbol_address(emitter, "x9", "_elephc_crypto_cipher_methods_fn");
    emitter.instruction("ldr x9, [x9]");                                        // load the published method-inventory entry
    emitter.instruction("cbz x9, __rt_openssl_methods_empty");                  // missing bridge produces an empty supported list
    abi::emit_call_reg(emitter, "x9");
    emitter.instruction("cmp x0, #0");                                          // negative values are stable bridge failures
    emitter.instruction("b.lt __rt_openssl_methods_empty");
    emitter.instruction("str x0, [sp, #520]");                                  // retain method count across array allocation
    emitter.instruction("mov x1, #16");                                         // string array element size = ptr + len
    abi::emit_call_label(emitter, "__rt_array_new");
    emitter.instruction("str x0, [sp, #528]");                                  // save the result array across pushes
    emitter.instruction("add x9, sp, #0");
    emitter.instruction("str x9, [sp, #536]");                                  // initialize packed-name cursor
    emitter.instruction("b __rt_openssl_methods_loop");

    emitter.label("__rt_openssl_methods_empty");
    emitter.instruction("mov x0, #0");                                          // create an empty string array on bridge failure
    emitter.instruction("mov x1, #16");
    abi::emit_call_label(emitter, "__rt_array_new");
    emitter.instruction("b __rt_openssl_methods_done");

    emitter.label("__rt_openssl_methods_loop");
    emitter.instruction("ldr x9, [sp, #512]");                                  // remaining packed bytes
    emitter.instruction("cbz x9, __rt_openssl_methods_finish");
    emitter.instruction("ldr x10, [sp, #536]");                                 // current name start
    emitter.instruction("mov x11, x10");                                        // scanning cursor
    emitter.label("__rt_openssl_methods_scan");
    emitter.instruction("ldrb w12, [x11], #1");                                 // read one packed-name byte
    emitter.instruction("sub x9, x9, #1");                                      // consume one packed byte
    emitter.instruction("cbnz w12, __rt_openssl_methods_scan");                 // stop at the trailing NUL
    emitter.instruction("sub x2, x11, x10");                                    // byte count including NUL
    emitter.instruction("sub x2, x2, #1");                                      // PHP string length excludes NUL
    emitter.instruction("str x9, [sp, #512]");                                  // save remaining packed byte count
    emitter.instruction("str x11, [sp, #536]");                                 // advance cursor to the next name
    emitter.instruction("ldr x0, [sp, #528]");                                  // reload result array
    emitter.instruction("mov x1, x10");                                         // name pointer
    abi::emit_call_label(emitter, "__rt_array_push_str");
    emitter.instruction("str x0, [sp, #528]");                                  // retain possibly-grown result array
    emitter.instruction("b __rt_openssl_methods_loop");

    emitter.label("__rt_openssl_methods_finish");
    emitter.instruction("ldr x0, [sp, #528]");                                  // return completed method array
    emitter.label("__rt_openssl_methods_done");
    emitter.instruction("add x9, sp, #544");                                    // address saved-frame area beyond pair-immediate range
    emitter.instruction("ldp x29, x30, [x9]");                                  // restore frame pointer and return address
    emitter.instruction("add sp, sp, #560");                                    // release helper frame
    emitter.instruction("ret");                                                 // return array handle in x0
}

/// Emits the Linux x86_64 method-inventory bridge and packed-name decoder.
fn emit_openssl_get_cipher_methods_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: openssl_get_cipher_methods ---");
    emitter.label_global("__rt_openssl_get_cipher_methods");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish stable local addressing
    emitter.instruction("sub rsp, 544");                                        // reserve 512-byte buffer plus parser state
    emitter.instruction("lea rsi, [rbp - 544]");                                // C arg1 = packed-name output buffer
    emitter.instruction("mov edx, 512");                                        // C arg2 = packed-name buffer capacity
    emitter.instruction("lea rcx, [rbp - 32]");                                 // C arg3 = packed output byte length
    abi::emit_load_symbol_to_reg(
        emitter,
        "r11",
        "_elephc_crypto_cipher_methods_fn",
        0,
    );
    emitter.instruction("test r11, r11");                                       // check whether the method bridge was published
    emitter.instruction("jz __rt_openssl_methods_empty_linux_x86_64");
    abi::emit_call_reg(emitter, "r11");
    emitter.instruction("test rax, rax");                                       // negative values are stable bridge failures
    emitter.instruction("js __rt_openssl_methods_empty_linux_x86_64");
    emitter.instruction("mov edi, eax");                                        // initial array capacity = method count
    emitter.instruction("mov esi, 16");                                         // string array element size = ptr + len
    abi::emit_call_label(emitter, "__rt_array_new");
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save result array across pushes
    emitter.instruction("lea r10, [rbp - 544]");
    emitter.instruction("mov QWORD PTR [rbp - 16], r10");                       // initialize packed-name cursor
    emitter.instruction("jmp __rt_openssl_methods_loop_linux_x86_64");

    emitter.label("__rt_openssl_methods_empty_linux_x86_64");
    emitter.instruction("xor edi, edi");                                        // create an empty string array on bridge failure
    emitter.instruction("mov esi, 16");
    abi::emit_call_label(emitter, "__rt_array_new");
    emitter.instruction("jmp __rt_openssl_methods_done_linux_x86_64");

    emitter.label("__rt_openssl_methods_loop_linux_x86_64");
    emitter.instruction("mov r9, QWORD PTR [rbp - 32]");                        // remaining packed bytes
    emitter.instruction("test r9, r9");
    emitter.instruction("jz __rt_openssl_methods_finish_linux_x86_64");
    emitter.instruction("mov r10, QWORD PTR [rbp - 16]");                       // current name start
    emitter.instruction("mov r11, r10");                                        // scanning cursor
    emitter.label("__rt_openssl_methods_scan_linux_x86_64");
    emitter.instruction("movzx eax, BYTE PTR [r11]");                           // read one packed-name byte
    emitter.instruction("add r11, 1");                                          // advance scanning cursor
    emitter.instruction("sub r9, 1");                                           // consume one packed byte
    emitter.instruction("test eax, eax");
    emitter.instruction("jnz __rt_openssl_methods_scan_linux_x86_64");          // stop at the trailing NUL
    emitter.instruction("mov rdx, r11");                                        // compute byte count including NUL
    emitter.instruction("sub rdx, r10");
    emitter.instruction("sub rdx, 1");                                          // PHP string length excludes NUL
    emitter.instruction("mov QWORD PTR [rbp - 32], r9");                        // save remaining packed byte count
    emitter.instruction("mov QWORD PTR [rbp - 16], r11");                       // advance cursor to next name
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // reload result array
    emitter.instruction("mov rsi, r10");                                        // name pointer
    abi::emit_call_label(emitter, "__rt_array_push_str");
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // retain possibly-grown result array
    emitter.instruction("jmp __rt_openssl_methods_loop_linux_x86_64");

    emitter.label("__rt_openssl_methods_finish_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // return completed method array
    emitter.label("__rt_openssl_methods_done_linux_x86_64");
    emitter.instruction("mov rsp, rbp");                                        // release packed buffer and parser state
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return array handle in rax
}
