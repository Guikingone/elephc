//! Purpose:
//! Emits the `__rt_get_http_response_headers` runtime helper, which builds an
//! indexed array of HTTP response header lines from `_http_resp_buf` for the
//! `$http_response_header` superglobal variable.
//!
//! Called from:
//! - The lowering of `$http_response_header` in the EIR backend.
//!
//! Key details:
//! - Reads `_http_resp_header_end` (the body start offset set by
//!   `__rt_http_open` after the CRLFCRLF scan) and splits
//!   `_http_resp_buf[0..header_end]` into lines at each `\r\n` boundary.
//! - Each line (including the status line) is pushed as an owned string into
//!   a new indexed array. Returns the array pointer.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// `__rt_get_http_response_headers() -> array`.
///
/// Builds an indexed array of HTTP response header lines from the buffered
/// response. Returns an empty array when no HTTP response has been received.
///
/// Output: x0/rax = indexed array pointer.
pub fn emit_get_http_response_headers(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_get_http_response_headers_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: get_http_response_headers ---");
    emitter.label_global("__rt_get_http_response_headers");

    // Frame (48 bytes):
    //   [sp, #0]  array pointer
    //   [sp, #8]  header end offset
    //   [sp, #16] scan index (current position in _http_resp_buf)
    //   [sp, #24] line start index
    //   [sp, #32] saved x29
    //   [sp, #40] saved x30
    emitter.instruction("sub sp, sp, #48");                                     // allocate the header-builder frame
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish the helper frame pointer

    // -- load the header end offset; 0 means no response received --
    abi::emit_symbol_address(emitter, "x9", "_http_resp_header_end");
    emitter.instruction("ldr x1, [x9]");                                        // load header_end
    emitter.instruction("str x1, [sp, #8]");                                    // save header_end
    emitter.instruction("cbz x1, __rt_http_headers_empty");                      // no headers → return empty array

    // -- create the result array (capacity 16) --
    emitter.instruction("mov x0, #16");                                         // initial capacity
    emitter.instruction("mov x1, #16");                                         // element size (ptr + len pair)
    emitter.instruction("bl __rt_array_new");                                   // x0 = array pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the array pointer

    // -- scan _http_resp_buf for CRLF line boundaries --
    emitter.instruction("mov x1, #0");                                          // scan index = 0
    emitter.instruction("mov x3, #0");                                          // line start = 0
    emitter.instruction("str x1, [sp, #16]");                                   // save scan index
    emitter.instruction("str x3, [sp, #24]");                                   // save line start
    emitter.label("__rt_http_headers_loop");
    emitter.instruction("ldr x1, [sp, #16]");                                   // reload scan index
    emitter.instruction("ldr x2, [sp, #8]");                                    // reload header_end
    emitter.instruction("cmp x1, x2");                                          // scan past the header region?
    emitter.instruction("b.ge __rt_http_headers_done");                          // yes → return the array

    // -- check for \r\n at the current position --
    abi::emit_symbol_address(emitter, "x4", "_http_resp_buf");
    emitter.instruction("ldrb w5, [x4, x1]");                                   // load byte at scan index
    emitter.instruction("cmp w5, #13");                                         // is it \r?
    emitter.instruction("b.ne __rt_http_headers_advance");                       // no → advance

    // -- found \r; check if \n follows --
    emitter.instruction("add x6, x1, #1");                                      // index of the next byte
    emitter.instruction("ldr x2, [sp, #8]");                                    // reload header_end
    emitter.instruction("cmp x6, x2");                                          // is \n within the header region?
    emitter.instruction("b.ge __rt_http_headers_push_last");                     // no → push the last line without \n

    abi::emit_symbol_address(emitter, "x4", "_http_resp_buf");
    emitter.instruction("ldrb w5, [x4, x6]");                                   // load the byte after \r
    emitter.instruction("cmp w5, #10");                                         // is it \n?
    emitter.instruction("b.ne __rt_http_headers_advance");                       // no → \r without \n, keep scanning

    // -- found \r\n: push the line from line_start to scan_index --
    emitter.instruction("ldr x3, [sp, #24]");                                   // reload line start
    emitter.instruction("sub x2, x1, x3");                                      // line length = scan_index - line_start
    abi::emit_symbol_address(emitter, "x4", "_http_resp_buf");
    emitter.instruction("add x1, x4, x3");                                      // line pointer = _http_resp_buf + line_start
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload array pointer
    emitter.instruction("bl __rt_array_push_str");                             // push the line; x0 = updated array
    emitter.instruction("str x0, [sp, #0]");                                    // save the updated array pointer

    // -- advance past \r\n --
    emitter.instruction("ldr x1, [sp, #16]");                                   // reload scan index
    emitter.instruction("add x1, x1, #2");                                      // skip past \r\n
    emitter.instruction("str x1, [sp, #16]");                                   // save the updated scan index
    emitter.instruction("str x1, [sp, #24]");                                   // line start = new scan index
    emitter.instruction("b __rt_http_headers_loop");                             // continue scanning

    emitter.label("__rt_http_headers_push_last");
    // -- push the final line (from line_start to header_end, no trailing \r\n) --
    emitter.instruction("ldr x3, [sp, #24]");                                   // reload line start
    emitter.instruction("ldr x2, [sp, #8]");                                    // reload header_end
    emitter.instruction("sub x2, x2, x3");                                      // line length = header_end - line_start
    abi::emit_symbol_address(emitter, "x4", "_http_resp_buf");
    emitter.instruction("add x1, x4, x3");                                      // line pointer = _http_resp_buf + line_start
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload array pointer
    emitter.instruction("bl __rt_array_push_str");                             // push the final line
    emitter.instruction("str x0, [sp, #0]");                                    // save the updated array pointer
    emitter.instruction("b __rt_http_headers_done");                             // done

    emitter.label("__rt_http_headers_advance");
    emitter.instruction("ldr x1, [sp, #16]");                                   // reload scan index
    emitter.instruction("add x1, x1, #1");                                      // advance by one byte
    emitter.instruction("str x1, [sp, #16]");                                   // save the updated scan index
    emitter.instruction("b __rt_http_headers_loop");                             // continue scanning

    emitter.label("__rt_http_headers_done");
    emitter.instruction("ldr x0, [sp, #0]");                                    // return the array pointer
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the frame
    emitter.instruction("ret");                                                 // return the array

    emitter.label("__rt_http_headers_empty");
    emitter.instruction("mov x0, #4");                                          // capacity 4 (min for new arrays)
    emitter.instruction("mov x1, #16");                                         // element size
    emitter.instruction("bl __rt_array_new");                                   // x0 = empty array
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the frame
    emitter.instruction("ret");                                                 // return the empty array
}

/// x86_64 Linux variant of `__rt_get_http_response_headers`.
fn emit_get_http_response_headers_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: get_http_response_headers ---");
    emitter.label_global("__rt_get_http_response_headers");

    // rbp-relative frame:
    //   [rbp - 8]  array pointer
    //   [rbp - 16] header end offset
    //   [rbp - 24] scan index
    //   [rbp - 32] line start
    emitter.instruction("push rbp");                                            // save caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish runtime frame pointer
    emitter.instruction("sub rsp, 32");                                         // reserve the frame

    // -- load the header end offset; 0 means no response received --
    abi::emit_symbol_address(emitter, "r9", "_http_resp_header_end");
    emitter.instruction("mov rcx, QWORD PTR [r9]");                             // load header_end
    emitter.instruction("mov QWORD PTR [rbp - 16], rcx");                        // save header_end
    emitter.instruction("test rcx, rcx");                                       // no headers?
    emitter.instruction("jz __rt_http_headers_empty_x86");                       // → return empty array

    // -- create the result array (capacity 16) --
    emitter.instruction("mov edi, 16");                                         // initial capacity
    emitter.instruction("mov esi, 16");                                         // element size
    emitter.instruction("call __rt_array_new");                                 // rax = array pointer
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the array pointer

    // -- scan _http_resp_buf for CRLF line boundaries --
    emitter.instruction("xor ecx, ecx");                                        // scan index = 0
    emitter.instruction("xor edx, edx");                                        // line start = 0
    emitter.instruction("mov QWORD PTR [rbp - 24], rcx");                        // save scan index
    emitter.instruction("mov QWORD PTR [rbp - 32], rdx");                        // save line start
    emitter.label("__rt_http_headers_loop_x86");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 24]");                        // reload scan index
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                        // reload header_end
    emitter.instruction("cmp rcx, rdx");                                        // scan past the header region?
    emitter.instruction("jge __rt_http_headers_done_x86");                       // yes → return the array

    // -- check for \r\n at the current position --
    abi::emit_symbol_address(emitter, "r10", "_http_resp_buf");
    emitter.instruction("movzx r11d, BYTE PTR [r10 + rcx]");                    // load byte at scan index
    emitter.instruction("cmp r11b, 13");                                        // is it \r?
    emitter.instruction("jne __rt_http_headers_advance_x86");                    // no → advance

    // -- found \r; check if \n follows --
    emitter.instruction("lea rax, [rcx + 1]");                                  // index of the next byte
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                        // reload header_end
    emitter.instruction("cmp rax, rdx");                                        // is \n within the header region?
    emitter.instruction("jge __rt_http_headers_push_last_x86");                   // no → push the last line

    abi::emit_symbol_address(emitter, "r10", "_http_resp_buf");
    emitter.instruction("movzx r11d, BYTE PTR [r10 + rax]");                    // load the byte after \r
    emitter.instruction("cmp r11b, 10");                                        // is it \n?
    emitter.instruction("jne __rt_http_headers_advance_x86");                    // no → \r without \n

    // -- found \r\n: push the line from line_start to scan_index --
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                        // reload line start
    emitter.instruction("sub rsi, rcx");                                        // line length = scan_index - line_start
    emitter.instruction("sub rsi, rdx");                                        // (fix: rsi = scan - line_start)
    // Actually: need rsi = scan_index - line_start. Let me compute properly.
    emitter.instruction("mov rsi, rcx");                                        // scan_index
    emitter.instruction("sub rsi, rdx");                                        // rsi = scan_index - line_start = line length
    abi::emit_symbol_address(emitter, "r10", "_http_resp_buf");
    emitter.instruction("lea rsi, [r10 + rdx]");                                // rsi = _http_resp_buf + line_start
    // Oops, I need both ptr and len. Let me use the correct registers.
    // __rt_array_push_str takes: rdi=array, rsi=ptr, rdx=len
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload array pointer
    emitter.instruction("mov rdx, rcx");                                        // scan_index
    emitter.instruction("sub rdx, QWORD PTR [rbp - 32]");                       // rdx = scan_index - line_start = line length
    // rsi already = _http_resp_buf + line_start (set above by lea)
    emitter.instruction("call __rt_array_push_str");                            // push the line; rax = updated array
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the updated array pointer

    // -- advance past \r\n --
    emitter.instruction("mov rcx, QWORD PTR [rbp - 24]");                        // reload scan index
    emitter.instruction("add rcx, 2");                                          // skip past \r\n
    emitter.instruction("mov QWORD PTR [rbp - 24], rcx");                        // save the updated scan index
    emitter.instruction("mov QWORD PTR [rbp - 32], rcx");                        // line start = new scan index
    emitter.instruction("jmp __rt_http_headers_loop_x86");                       // continue scanning

    emitter.label("__rt_http_headers_push_last_x86");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                        // reload line start
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                        // reload header_end
    emitter.instruction("sub rcx, rdx");                                        // rcx = header_end - line_start = line length
    abi::emit_symbol_address(emitter, "r10", "_http_resp_buf");
    emitter.instruction("lea rsi, [r10 + rdx]");                                // rsi = _http_resp_buf + line_start
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload array pointer
    emitter.instruction("mov rdx, rcx");                                        // rdx = line length
    emitter.instruction("call __rt_array_push_str");                            // push the final line
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the updated array pointer
    emitter.instruction("jmp __rt_http_headers_done_x86");                       // done

    emitter.label("__rt_http_headers_advance_x86");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 24]");                        // reload scan index
    emitter.instruction("add rcx, 1");                                          // advance by one byte
    emitter.instruction("mov QWORD PTR [rbp - 24], rcx");                        // save the updated scan index
    emitter.instruction("jmp __rt_http_headers_loop_x86");                       // continue scanning

    emitter.label("__rt_http_headers_done_x86");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // return the array pointer
    emitter.instruction("leave");                                               // restore rbp + rsp
    emitter.instruction("ret");                                                 // return the array

    emitter.label("__rt_http_headers_empty_x86");
    emitter.instruction("mov edi, 4");                                          // capacity 4
    emitter.instruction("mov esi, 16");                                         // element size
    emitter.instruction("call __rt_array_new");                                 // rax = empty array
    emitter.instruction("leave");                                               // restore rbp + rsp
    emitter.instruction("ret");                                                 // return the empty array
}