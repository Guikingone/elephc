//! Purpose:
//! Emits OpenSSL encrypt/decrypt runtime glue around the elephc-crypto bridge.
//!
//! Called from:
//! - `runtime::emitters::emit_runtime()` through the string-runtime module.
//!
//! Key details:
//! - Callers pass a target-neutral field block so the helpers own the large C ABI calls.
//! - The bridge always sees raw bytes; this glue applies PHP's base64 option semantics.
//! - Phase 2 supplies no GCM tag output/input, so AEAD calls return false until phase 3.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the encrypt and decrypt helpers for the active target.
pub fn emit_openssl_cipher(emitter: &mut Emitter) {
    emit_openssl_encrypt(emitter);
    emit_openssl_decrypt(emitter);
}

/// Emits `__rt_openssl_encrypt`, returning an owned string or a null failure pointer.
fn emit_openssl_encrypt(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_openssl_encrypt_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: openssl_encrypt ---");
    emitter.label_global("__rt_openssl_encrypt");
    emitter.instruction("sub sp, sp, #160");                                    // reserve C stack args, local state, and saved frame
    emitter.instruction("stp x29, x30, [sp, #144]");                            // preserve caller frame and return address
    emitter.instruction("add x29, sp, #144");                                   // establish stable helper frame
    emitter.instruction("str x0, [sp, #80]");                                   // retain caller field-block pointer
    emitter.instruction("ldr x9, [x0, #8]");                                    // plaintext byte length
    emitter.instruction("add x9, x9, #16");                                     // CBC/ECB PKCS#7 needs at most one extra block
    emitter.instruction("str x9, [sp, #96]");                                   // save bridge output capacity
    emitter.instruction("mov x0, x9");                                          // heap allocation byte count
    abi::emit_call_label(emitter, "__rt_heap_alloc");
    emitter.instruction("mov x9, #1");                                          // heap kind 1 = owned string
    emitter.instruction("str x9, [x0, #-8]");                                   // stamp raw ciphertext allocation
    emitter.instruction("str x0, [sp, #88]");                                   // retain raw output pointer
    emitter.instruction("str xzr, [sp, #104]");                                 // initialize ciphertext output length
    emitter.instruction("str xzr, [sp, #112]");                                 // initialize ignored tag output length

    emitter.instruction("ldr x10, [sp, #80]");                                  // reload caller field block
    emitter.instruction("ldr x9, [x10, #48]");                                  // options
    emitter.instruction("str x9, [sp, #0]");                                    // C stack arg8 = options
    emitter.instruction("ldr x9, [x10, #72]");                                  // AAD pointer
    emitter.instruction("str x9, [sp, #8]");                                    // C stack arg9 = AAD pointer
    emitter.instruction("ldr x9, [x10, #80]");                                  // AAD length
    emitter.instruction("str x9, [sp, #16]");                                   // C stack arg10 = AAD length
    emitter.instruction("ldr x9, [x10, #88]");                                  // requested tag length
    emitter.instruction("str x9, [sp, #24]");                                   // C stack arg11 = tag length
    emitter.instruction("ldr x9, [sp, #88]");                                   // raw ciphertext destination
    emitter.instruction("str x9, [sp, #32]");                                   // C stack arg12 = output pointer
    emitter.instruction("ldr x9, [sp, #96]");                                   // raw ciphertext capacity
    emitter.instruction("str x9, [sp, #40]");                                   // C stack arg13 = output capacity
    emitter.instruction("add x9, sp, #104");
    emitter.instruction("str x9, [sp, #48]");                                   // C stack arg14 = output length pointer
    emitter.instruction("str xzr, [sp, #56]");                                  // C stack arg15 = no phase-2 tag buffer
    emitter.instruction("str xzr, [sp, #64]");                                  // C stack arg16 = zero tag capacity
    emitter.instruction("add x9, sp, #112");
    emitter.instruction("str x9, [sp, #72]");                                   // C stack arg17 = tag length pointer

    emitter.instruction("ldr x0, [x10, #16]");                                  // C arg0 = cipher-name pointer
    emitter.instruction("ldr x1, [x10, #24]");                                  // C arg1 = cipher-name length
    emitter.instruction("ldr x2, [x10, #0]");                                   // C arg2 = plaintext pointer
    emitter.instruction("ldr x3, [x10, #8]");                                   // C arg3 = plaintext length
    emitter.instruction("ldr x4, [x10, #32]");                                  // C arg4 = passphrase pointer
    emitter.instruction("ldr x5, [x10, #40]");                                  // C arg5 = passphrase length
    emitter.instruction("ldr x6, [x10, #56]");                                  // C arg6 = IV pointer
    emitter.instruction("ldr x7, [x10, #64]");                                  // C arg7 = IV length
    abi::emit_symbol_address(emitter, "x9", "_elephc_crypto_encrypt_fn");
    emitter.instruction("ldr x9, [x9]");                                        // load published encryption bridge entry
    emitter.instruction("cbz x9, __rt_openssl_encrypt_fail");                   // missing bridge returns PHP false
    abi::emit_call_reg(emitter, "x9");
    emitter.instruction("cbnz x0, __rt_openssl_encrypt_fail");                  // nonzero bridge status returns PHP false

    emitter.instruction("ldr x10, [sp, #80]");                                  // recover field block after C call
    emitter.instruction("ldr x9, [x10, #48]");                                  // inspect OPENSSL_RAW_DATA
    emitter.instruction("tbnz x9, #0, __rt_openssl_encrypt_raw");
    emitter.instruction("ldr x1, [sp, #88]");                                   // base64 input = raw ciphertext pointer
    emitter.instruction("ldr x2, [sp, #104]");                                  // base64 input length
    abi::emit_call_label(emitter, "__rt_base64_encode");
    abi::emit_call_label(emitter, "__rt_str_persist");                         // own the encoded result beyond concat scratch
    emitter.instruction("str x1, [sp, #120]");                                  // save owned base64 pointer across raw free
    emitter.instruction("str x2, [sp, #128]");                                  // save owned base64 length across raw free
    emitter.instruction("ldr x0, [sp, #88]");                                   // release intermediate raw ciphertext allocation
    abi::emit_call_label(emitter, "__rt_heap_free_safe");
    emitter.instruction("ldr x1, [sp, #120]");                                  // restore owned base64 result pointer
    emitter.instruction("ldr x2, [sp, #128]");                                  // restore owned base64 result length
    emitter.instruction("b __rt_openssl_encrypt_done");

    emitter.label("__rt_openssl_encrypt_raw");
    emitter.instruction("ldr x1, [sp, #88]");                                   // return owned raw ciphertext pointer
    emitter.instruction("ldr x2, [sp, #104]");                                  // return raw ciphertext length
    emitter.instruction("b __rt_openssl_encrypt_done");

    emitter.label("__rt_openssl_encrypt_fail");
    emitter.instruction("ldr x0, [sp, #88]");                                   // release unused raw output allocation
    abi::emit_call_label(emitter, "__rt_heap_free_safe");
    emitter.instruction("mov x1, #0");                                          // null pointer is the string|false failure sentinel
    emitter.instruction("mov x2, #0");                                          // clear failure length

    emitter.label("__rt_openssl_encrypt_done");
    emitter.instruction("ldp x29, x30, [sp, #144]");                            // restore caller frame and return address
    emitter.instruction("add sp, sp, #160");                                    // release helper frame
    emitter.instruction("ret");                                                 // return owned string pair or null sentinel
}

/// Emits the Linux x86_64 encryption helper and its 18-argument SysV bridge call.
fn emit_openssl_encrypt_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: openssl_encrypt ---");
    emitter.label_global("__rt_openssl_encrypt");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish stable local addressing
    emitter.instruction("sub rsp, 160");                                        // reserve 96-byte outgoing area plus local state
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // retain caller field-block pointer
    emitter.instruction("mov r9, QWORD PTR [rdi + 8]");                         // plaintext byte length
    emitter.instruction("add r9, 16");                                          // CBC/ECB PKCS#7 needs at most one extra block
    emitter.instruction("mov QWORD PTR [rbp - 24], r9");                        // save bridge output capacity
    emitter.instruction("mov rax, r9");                                         // heap allocation byte count
    abi::emit_call_label(emitter, "__rt_heap_alloc");
    emitter.instruction(&format!(
        "mov r10, 0x{:x}",
        crate::codegen_support::sentinels::x86_64_heap_kind_word(1)
    ));
    emitter.instruction("mov QWORD PTR [rax - 8], r10");                        // stamp raw ciphertext as owned string
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // retain raw output pointer
    emitter.instruction("mov QWORD PTR [rbp - 32], 0");                         // initialize ciphertext output length
    emitter.instruction("mov QWORD PTR [rbp - 40], 0");                         // initialize ignored tag output length

    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload caller field block
    emitter.instruction("mov r11, QWORD PTR [r10 + 56]");
    emitter.instruction("mov QWORD PTR [rsp + 0], r11");                        // C stack arg6 = IV pointer
    emitter.instruction("mov r11, QWORD PTR [r10 + 64]");
    emitter.instruction("mov QWORD PTR [rsp + 8], r11");                        // C stack arg7 = IV length
    emitter.instruction("mov r11, QWORD PTR [r10 + 48]");
    emitter.instruction("mov QWORD PTR [rsp + 16], r11");                       // C stack arg8 = options
    emitter.instruction("mov r11, QWORD PTR [r10 + 72]");
    emitter.instruction("mov QWORD PTR [rsp + 24], r11");                       // C stack arg9 = AAD pointer
    emitter.instruction("mov r11, QWORD PTR [r10 + 80]");
    emitter.instruction("mov QWORD PTR [rsp + 32], r11");                       // C stack arg10 = AAD length
    emitter.instruction("mov r11, QWORD PTR [r10 + 88]");
    emitter.instruction("mov QWORD PTR [rsp + 40], r11");                       // C stack arg11 = tag length
    emitter.instruction("mov r11, QWORD PTR [rbp - 16]");
    emitter.instruction("mov QWORD PTR [rsp + 48], r11");                       // C stack arg12 = output pointer
    emitter.instruction("mov r11, QWORD PTR [rbp - 24]");
    emitter.instruction("mov QWORD PTR [rsp + 56], r11");                       // C stack arg13 = output capacity
    emitter.instruction("lea r11, [rbp - 32]");
    emitter.instruction("mov QWORD PTR [rsp + 64], r11");                       // C stack arg14 = output length pointer
    emitter.instruction("mov QWORD PTR [rsp + 72], 0");                         // C stack arg15 = no phase-2 tag buffer
    emitter.instruction("mov QWORD PTR [rsp + 80], 0");                         // C stack arg16 = zero tag capacity
    emitter.instruction("lea r11, [rbp - 40]");
    emitter.instruction("mov QWORD PTR [rsp + 88], r11");                       // C stack arg17 = tag length pointer
    emitter.instruction("mov rdi, QWORD PTR [r10 + 16]");                       // C arg0 = cipher-name pointer
    emitter.instruction("mov rsi, QWORD PTR [r10 + 24]");                       // C arg1 = cipher-name length
    emitter.instruction("mov rdx, QWORD PTR [r10 + 0]");                        // C arg2 = plaintext pointer
    emitter.instruction("mov rcx, QWORD PTR [r10 + 8]");                        // C arg3 = plaintext length
    emitter.instruction("mov r8, QWORD PTR [r10 + 32]");                        // C arg4 = passphrase pointer
    emitter.instruction("mov r9, QWORD PTR [r10 + 40]");                        // C arg5 = passphrase length
    abi::emit_load_symbol_to_reg(emitter, "r11", "_elephc_crypto_encrypt_fn", 0);
    emitter.instruction("test r11, r11");                                       // check whether encryption bridge was published
    emitter.instruction("jz __rt_openssl_encrypt_fail_linux_x86_64");
    abi::emit_call_reg(emitter, "r11");
    emitter.instruction("test eax, eax");                                       // zero is the bridge success status
    emitter.instruction("jnz __rt_openssl_encrypt_fail_linux_x86_64");

    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // recover field block after C call
    emitter.instruction("test QWORD PTR [r10 + 48], 1");                        // inspect OPENSSL_RAW_DATA
    emitter.instruction("jnz __rt_openssl_encrypt_raw_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // base64 input = raw ciphertext pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                       // base64 input length
    abi::emit_call_label(emitter, "__rt_base64_encode");
    abi::emit_call_label(emitter, "__rt_str_persist");                         // own encoded result beyond concat scratch
    emitter.instruction("mov QWORD PTR [rbp - 48], rax");                       // save owned base64 pointer across raw free
    emitter.instruction("mov QWORD PTR [rbp - 56], rdx");                       // save owned base64 length across raw free
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // release intermediate raw ciphertext
    abi::emit_call_label(emitter, "__rt_heap_free_safe");
    emitter.instruction("mov rax, QWORD PTR [rbp - 48]");                       // restore owned base64 result pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 56]");                       // restore owned base64 result length
    emitter.instruction("jmp __rt_openssl_encrypt_done_linux_x86_64");

    emitter.label("__rt_openssl_encrypt_raw_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // return owned raw ciphertext pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                       // return raw ciphertext length
    emitter.instruction("jmp __rt_openssl_encrypt_done_linux_x86_64");

    emitter.label("__rt_openssl_encrypt_fail_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // release unused raw output allocation
    abi::emit_call_label(emitter, "__rt_heap_free_safe");
    emitter.instruction("xor eax, eax");                                        // null pointer is the string|false failure sentinel
    emitter.instruction("xor edx, edx");                                        // clear failure length

    emitter.label("__rt_openssl_encrypt_done_linux_x86_64");
    emitter.instruction("mov rsp, rbp");                                        // release outgoing stack area and locals
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return owned string pair or null sentinel
}

/// Emits `__rt_openssl_decrypt`, decoding base64 before the raw bridge call when needed.
fn emit_openssl_decrypt(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_openssl_decrypt_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: openssl_decrypt ---");
    emitter.label_global("__rt_openssl_decrypt");
    emitter.instruction("sub sp, sp, #144");                                    // reserve C stack args, local state, and saved frame
    emitter.instruction("stp x29, x30, [sp, #128]");                            // preserve caller frame and return address
    emitter.instruction("add x29, sp, #128");                                   // establish stable helper frame
    emitter.instruction("str x0, [sp, #64]");                                   // retain caller field-block pointer
    emitter.instruction("ldr x9, [x0, #48]");                                   // inspect OPENSSL_RAW_DATA
    emitter.instruction("tbnz x9, #0, __rt_openssl_decrypt_raw_input");
    emitter.instruction("ldr x1, [x0, #0]");                                    // encoded ciphertext pointer
    emitter.instruction("ldr x2, [x0, #8]");                                    // encoded ciphertext length
    abi::emit_call_label(emitter, "__rt_base64_decode");
    emitter.instruction("str x1, [sp, #96]");                                   // retain decoded ciphertext pointer
    emitter.instruction("str x2, [sp, #104]");                                  // retain decoded ciphertext length
    emitter.instruction("b __rt_openssl_decrypt_input_ready");
    emitter.label("__rt_openssl_decrypt_raw_input");
    emitter.instruction("ldr x10, [sp, #64]");                                  // reload field block
    emitter.instruction("ldr x9, [x10, #0]");
    emitter.instruction("str x9, [sp, #96]");                                   // raw ciphertext pointer
    emitter.instruction("ldr x9, [x10, #8]");
    emitter.instruction("str x9, [sp, #104]");                                  // raw ciphertext length
    emitter.label("__rt_openssl_decrypt_input_ready");

    emitter.instruction("ldr x9, [sp, #104]");                                  // plaintext capacity is at most ciphertext length
    emitter.instruction("cmp x9, #1");
    emitter.instruction("mov x10, #1");
    emitter.instruction("csel x9, x9, x10, hs");                                // allocate a non-null owner for empty plaintext
    emitter.instruction("str x9, [sp, #80]");                                   // save output capacity
    emitter.instruction("mov x0, x9");
    abi::emit_call_label(emitter, "__rt_heap_alloc");
    emitter.instruction("mov x9, #1");                                          // heap kind 1 = owned string
    emitter.instruction("str x9, [x0, #-8]");                                   // stamp plaintext allocation
    emitter.instruction("str x0, [sp, #72]");                                   // retain plaintext pointer
    emitter.instruction("str xzr, [sp, #88]");                                  // initialize plaintext output length

    emitter.instruction("ldr x10, [sp, #64]");                                  // reload caller field block
    emitter.instruction("ldr x9, [x10, #48]");
    emitter.instruction("str x9, [sp, #0]");                                    // C stack arg8 = options
    emitter.instruction("ldr x9, [x10, #72]");
    emitter.instruction("str x9, [sp, #8]");                                    // C stack arg9 = AAD pointer
    emitter.instruction("ldr x9, [x10, #80]");
    emitter.instruction("str x9, [sp, #16]");                                   // C stack arg10 = AAD length
    emitter.instruction("ldr x9, [x10, #88]");
    emitter.instruction("str x9, [sp, #24]");                                   // C stack arg11 = tag pointer
    emitter.instruction("ldr x9, [x10, #96]");
    emitter.instruction("str x9, [sp, #32]");                                   // C stack arg12 = tag length
    emitter.instruction("ldr x9, [sp, #72]");
    emitter.instruction("str x9, [sp, #40]");                                   // C stack arg13 = output pointer
    emitter.instruction("ldr x9, [sp, #80]");
    emitter.instruction("str x9, [sp, #48]");                                   // C stack arg14 = output capacity
    emitter.instruction("add x9, sp, #88");
    emitter.instruction("str x9, [sp, #56]");                                   // C stack arg15 = output length pointer
    emitter.instruction("ldr x0, [x10, #16]");                                  // C arg0 = cipher-name pointer
    emitter.instruction("ldr x1, [x10, #24]");                                  // C arg1 = cipher-name length
    emitter.instruction("ldr x2, [sp, #96]");                                   // C arg2 = raw ciphertext pointer
    emitter.instruction("ldr x3, [sp, #104]");                                  // C arg3 = raw ciphertext length
    emitter.instruction("ldr x4, [x10, #32]");                                  // C arg4 = passphrase pointer
    emitter.instruction("ldr x5, [x10, #40]");                                  // C arg5 = passphrase length
    emitter.instruction("ldr x6, [x10, #56]");                                  // C arg6 = IV pointer
    emitter.instruction("ldr x7, [x10, #64]");                                  // C arg7 = IV length
    abi::emit_symbol_address(emitter, "x9", "_elephc_crypto_decrypt_fn");
    emitter.instruction("ldr x9, [x9]");                                        // load published decryption bridge entry
    emitter.instruction("cbz x9, __rt_openssl_decrypt_fail");                   // missing bridge returns PHP false
    abi::emit_call_reg(emitter, "x9");
    emitter.instruction("cbnz x0, __rt_openssl_decrypt_fail");                  // nonzero bridge status returns PHP false
    emitter.instruction("ldr x1, [sp, #72]");                                   // return owned plaintext pointer
    emitter.instruction("ldr x2, [sp, #88]");                                   // return plaintext length
    emitter.instruction("b __rt_openssl_decrypt_done");

    emitter.label("__rt_openssl_decrypt_fail");
    emitter.instruction("ldr x0, [sp, #72]");                                   // release unused plaintext allocation
    abi::emit_call_label(emitter, "__rt_heap_free_safe");
    emitter.instruction("mov x1, #0");                                          // null pointer is the string|false failure sentinel
    emitter.instruction("mov x2, #0");                                          // clear failure length
    emitter.label("__rt_openssl_decrypt_done");
    emitter.instruction("ldp x29, x30, [sp, #128]");                            // restore caller frame and return address
    emitter.instruction("add sp, sp, #144");                                    // release helper frame
    emitter.instruction("ret");                                                 // return owned string pair or null sentinel
}

/// Emits the Linux x86_64 decryption helper and its 16-argument SysV bridge call.
fn emit_openssl_decrypt_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: openssl_decrypt ---");
    emitter.label_global("__rt_openssl_decrypt");
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish stable local addressing
    emitter.instruction("sub rsp, 144");                                        // reserve 80-byte outgoing area plus local state
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // retain caller field-block pointer
    emitter.instruction("test QWORD PTR [rdi + 48], 1");                        // inspect OPENSSL_RAW_DATA
    emitter.instruction("jnz __rt_openssl_decrypt_raw_input_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rdi + 0]");                        // encoded ciphertext pointer
    emitter.instruction("mov rdx, QWORD PTR [rdi + 8]");                        // encoded ciphertext length
    abi::emit_call_label(emitter, "__rt_base64_decode");
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // retain decoded ciphertext pointer
    emitter.instruction("mov QWORD PTR [rbp - 48], rdx");                       // retain decoded ciphertext length
    emitter.instruction("jmp __rt_openssl_decrypt_input_ready_linux_x86_64");
    emitter.label("__rt_openssl_decrypt_raw_input_linux_x86_64");
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload field block
    emitter.instruction("mov r9, QWORD PTR [r10 + 0]");
    emitter.instruction("mov QWORD PTR [rbp - 40], r9");                        // raw ciphertext pointer
    emitter.instruction("mov r9, QWORD PTR [r10 + 8]");
    emitter.instruction("mov QWORD PTR [rbp - 48], r9");                        // raw ciphertext length
    emitter.label("__rt_openssl_decrypt_input_ready_linux_x86_64");

    emitter.instruction("mov r9, QWORD PTR [rbp - 48]");                        // plaintext capacity is at most ciphertext length
    emitter.instruction("cmp r9, 1");
    emitter.instruction("jae __rt_openssl_decrypt_capacity_linux_x86_64");
    emitter.instruction("mov r9, 1");                                           // allocate a non-null owner for empty plaintext
    emitter.label("__rt_openssl_decrypt_capacity_linux_x86_64");
    emitter.instruction("mov QWORD PTR [rbp - 24], r9");                        // save output capacity
    emitter.instruction("mov rax, r9");
    abi::emit_call_label(emitter, "__rt_heap_alloc");
    emitter.instruction(&format!(
        "mov r10, 0x{:x}",
        crate::codegen_support::sentinels::x86_64_heap_kind_word(1)
    ));
    emitter.instruction("mov QWORD PTR [rax - 8], r10");                        // stamp plaintext as owned string
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // retain plaintext pointer
    emitter.instruction("mov QWORD PTR [rbp - 32], 0");                         // initialize plaintext output length

    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload caller field block
    emitter.instruction("mov r11, QWORD PTR [r10 + 56]");
    emitter.instruction("mov QWORD PTR [rsp + 0], r11");                        // C stack arg6 = IV pointer
    emitter.instruction("mov r11, QWORD PTR [r10 + 64]");
    emitter.instruction("mov QWORD PTR [rsp + 8], r11");                        // C stack arg7 = IV length
    emitter.instruction("mov r11, QWORD PTR [r10 + 48]");
    emitter.instruction("mov QWORD PTR [rsp + 16], r11");                       // C stack arg8 = options
    emitter.instruction("mov r11, QWORD PTR [r10 + 72]");
    emitter.instruction("mov QWORD PTR [rsp + 24], r11");                       // C stack arg9 = AAD pointer
    emitter.instruction("mov r11, QWORD PTR [r10 + 80]");
    emitter.instruction("mov QWORD PTR [rsp + 32], r11");                       // C stack arg10 = AAD length
    emitter.instruction("mov r11, QWORD PTR [r10 + 88]");
    emitter.instruction("mov QWORD PTR [rsp + 40], r11");                       // C stack arg11 = tag pointer
    emitter.instruction("mov r11, QWORD PTR [r10 + 96]");
    emitter.instruction("mov QWORD PTR [rsp + 48], r11");                       // C stack arg12 = tag length
    emitter.instruction("mov r11, QWORD PTR [rbp - 16]");
    emitter.instruction("mov QWORD PTR [rsp + 56], r11");                       // C stack arg13 = output pointer
    emitter.instruction("mov r11, QWORD PTR [rbp - 24]");
    emitter.instruction("mov QWORD PTR [rsp + 64], r11");                       // C stack arg14 = output capacity
    emitter.instruction("lea r11, [rbp - 32]");
    emitter.instruction("mov QWORD PTR [rsp + 72], r11");                       // C stack arg15 = output length pointer
    emitter.instruction("mov rdi, QWORD PTR [r10 + 16]");                       // C arg0 = cipher-name pointer
    emitter.instruction("mov rsi, QWORD PTR [r10 + 24]");                       // C arg1 = cipher-name length
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // C arg2 = raw ciphertext pointer
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // C arg3 = raw ciphertext length
    emitter.instruction("mov r8, QWORD PTR [r10 + 32]");                        // C arg4 = passphrase pointer
    emitter.instruction("mov r9, QWORD PTR [r10 + 40]");                        // C arg5 = passphrase length
    abi::emit_load_symbol_to_reg(emitter, "r11", "_elephc_crypto_decrypt_fn", 0);
    emitter.instruction("test r11, r11");                                       // check whether decryption bridge was published
    emitter.instruction("jz __rt_openssl_decrypt_fail_linux_x86_64");
    abi::emit_call_reg(emitter, "r11");
    emitter.instruction("test eax, eax");                                       // zero is the bridge success status
    emitter.instruction("jnz __rt_openssl_decrypt_fail_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // return owned plaintext pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                       // return plaintext length
    emitter.instruction("jmp __rt_openssl_decrypt_done_linux_x86_64");

    emitter.label("__rt_openssl_decrypt_fail_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // release unused plaintext allocation
    abi::emit_call_label(emitter, "__rt_heap_free_safe");
    emitter.instruction("xor eax, eax");                                        // null pointer is the string|false failure sentinel
    emitter.instruction("xor edx, edx");                                        // clear failure length
    emitter.label("__rt_openssl_decrypt_done_linux_x86_64");
    emitter.instruction("mov rsp, rbp");                                        // release outgoing stack area and locals
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return owned string pair or null sentinel
}
