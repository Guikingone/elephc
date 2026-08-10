//! Purpose:
//! Stream close, read, write, formatted IO, and CSV calls.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::io`.
//!
//! Key details:
//! - Preserves target-aware ABI handling, runtime calls, and result ownership.

use super::*;

/// Lowers `fclose(stream)` after validating and unboxing the stream handle.
pub(crate) fn lower_fclose(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "fclose", 1)?;
    let stream = expect_operand(inst, 0)?;
    begin_stream_close(ctx, stream, "fclose")?;
    let success_label = ctx.next_label("fclose_ok");
    let done_label = ctx.next_label("fclose_done");
    let user_wrapper_label = ctx.next_label("fclose_user_wrapper");
    let phar_label = ctx.next_label("fclose_phar");
    let not_phar_label = ctx.next_label("fclose_not_phar");
    let after_dispatch_label = ctx.next_label("fclose_after_dispatch");
    let not_popen_label = ctx.next_label("fclose_not_popen");
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("ldr x0, [sp, #16]");                       // resolve the opaque handle while preserving its descriptor
            abi::emit_call_label(ctx.emitter, "__rt_stream_state");
            // PHP invalidates attached filter resources at fclose(), not when the
            // last reference to the stream goes away, so the chains are closed here
            // as well as from the state destructor.
            ctx.emitter.instruction("stp x0, x1, [sp, #-16]!");                 // preserve the resolved state across the teardown call
            abi::emit_call_label(ctx.emitter, "__rt_stream_close_filter_chains");
            ctx.emitter.instruction("ldp x0, x1, [sp], #16");                   // restore the resolved state
            ctx.emitter.instruction(&format!("cbz x0, {}", not_popen_label));   // retain defensive descriptor cleanup if state vanished
            ctx.emitter.instruction(&format!(
                "ldr x9, [x0, #{}]", STREAM_BACKEND_KIND_OFFSET
            ));                                                                 // select cleanup from the authoritative StreamState backend
            ctx.emitter.instruction(&format!("cmp x9, #{}", STREAM_BACKEND_POPEN)); // is this stream owned by libc popen?
            ctx.emitter.instruction(&format!("b.ne {}", not_popen_label));      // ordinary streams retain their existing flush and close path
            ctx.emitter.instruction(&format!(
                "ldr x10, [x0, #{}]", STREAM_BACKEND_AUX_OFFSET
            ));                                                                 // load the owning FILE* independently of the reusable fd
            ctx.emitter.instruction(&format!(
                "str xzr, [x0, #{}]", STREAM_BACKEND_AUX_OFFSET
            ));                                                                 // detach process ownership before re-entrant libc cleanup
            ctx.emitter.instruction("mov x0, x10");                             // pass the owning FILE* to pclose
            abi::emit_call_label(ctx.emitter, "__rt_pclose");
            ctx.emitter.instruction("cmn x0, #1");                              // did pclose report its -1 failure sentinel?
            ctx.emitter.instruction("cset x0, ne");                             // fclose(process pipe) returns a PHP boolean
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            ctx.emitter.instruction(&format!("b {}", after_dispatch_label));    // skip descriptor-only cleanup after pclose
            ctx.emitter.label(&not_popen_label);
            abi::emit_pop_reg(ctx.emitter, "x0");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 16]");           // resolve the opaque handle while preserving its descriptor
            abi::emit_call_label(ctx.emitter, "__rt_stream_state");
            // PHP invalidates attached filter resources at fclose(), not when the
            // last reference to the stream goes away, so the chains are closed here
            // as well as from the state destructor.
            ctx.emitter.instruction("push rax");                                // preserve the resolved state across the teardown call
            ctx.emitter.instruction("mov rdi, rax");
            abi::emit_call_label(ctx.emitter, "__rt_stream_close_filter_chains");
            ctx.emitter.instruction("pop rax");                                 // restore the resolved state
            ctx.emitter.instruction("test rax, rax");                           // did the Closing StreamState resolve?
            ctx.emitter.instruction(&format!("jz {}", not_popen_label));        // retain defensive descriptor cleanup if state vanished
            ctx.emitter.instruction(&format!(
                "mov r9, QWORD PTR [rax + {}]", STREAM_BACKEND_KIND_OFFSET
            ));                                                                 // select cleanup from the authoritative StreamState backend
            ctx.emitter.instruction(&format!("cmp r9, {}", STREAM_BACKEND_POPEN)); // is this stream owned by libc popen?
            ctx.emitter.instruction(&format!("jne {}", not_popen_label));       // ordinary streams retain their existing flush and close path
            ctx.emitter.instruction(&format!(
                "mov rdi, QWORD PTR [rax + {}]", STREAM_BACKEND_AUX_OFFSET
            ));                                                                 // load the owning FILE* independently of the reusable fd
            ctx.emitter.instruction(&format!(
                "mov QWORD PTR [rax + {}], 0", STREAM_BACKEND_AUX_OFFSET
            ));                                                                 // detach process ownership before re-entrant libc cleanup
            abi::emit_call_label(ctx.emitter, "__rt_pclose");
            ctx.emitter.instruction("cmp eax, -1");                             // did pclose report its failure sentinel?
            ctx.emitter.instruction("setne al");                                // fclose(process pipe) returns a PHP boolean
            ctx.emitter.instruction("movzx eax, al");                           // widen the strict boolean close result
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            ctx.emitter.instruction(&format!("jmp {}", after_dispatch_label));  // skip descriptor-only cleanup after pclose
            ctx.emitter.label(&not_popen_label);
            abi::emit_pop_reg(ctx.emitter, "rax");
        }
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov w9, #0x5000");                         // low half of the phar-write descriptor base 0x50000000
            ctx.emitter.instruction("lsl w9, w9, #16");                         // form the phar-write synthetic descriptor base
            ctx.emitter.instruction("cmp x0, x9");                              // is the descriptor below the phar-write range?
            ctx.emitter.instruction(&format!("b.lt {}", not_phar_label));       // below the PHAR range: continue with normal dispatch
            ctx.emitter.instruction("add x10, x9, #32");                        // upper bound for the 32 buffered PHAR write descriptors
            ctx.emitter.instruction("cmp x0, x10");                             // is this inside the phar-write descriptor range?
            ctx.emitter.instruction(&format!("b.lt {}", phar_label));           // finalize phar writes instead of closing a real fd
            ctx.emitter.label(&not_phar_label);
            ctx.emitter.instruction("mov w9, #0x4000");                         // materialize the high half of USER_WRAPPER_FD_BASE
            ctx.emitter.instruction("lsl w9, w9, #16");                         // form the synthetic wrapper fd base 0x40000000
            ctx.emitter.instruction("cmp x0, x9");                              // test whether this is a userspace-wrapper stream
            ctx.emitter.instruction(&format!("b.ge {}", user_wrapper_label));   // dispatch synthetic handles without indexing fd tables
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov r9d, 0x50000000");                     // materialize the phar-write synthetic descriptor base
            ctx.emitter.instruction("cmp rax, r9");                             // is the descriptor below the phar-write range?
            ctx.emitter.instruction(&format!("jl {}", not_phar_label));         // below the PHAR range: continue with normal dispatch
            ctx.emitter.instruction("lea r10, [r9 + 32]");                      // upper bound for the 32 buffered PHAR write descriptors
            ctx.emitter.instruction("cmp rax, r10");                            // is this inside the phar-write descriptor range?
            ctx.emitter.instruction(&format!("jl {}", phar_label));             // finalize phar writes instead of closing a real fd
            ctx.emitter.label(&not_phar_label);
            ctx.emitter.instruction("mov r9d, 0x40000000");                     // materialize USER_WRAPPER_FD_BASE for synthetic handles
            ctx.emitter.instruction("cmp rax, r9");                             // test whether this is a userspace-wrapper stream
            ctx.emitter.instruction(&format!("jge {}", user_wrapper_label));    // dispatch synthetic handles without indexing fd tables
        }
    }
    emit_zlib_flush_on_close_for_current_fd(ctx);
    emit_bz2_flush_on_close_for_current_fd(ctx);
    emit_iconv_flush_on_close_for_current_fd(ctx);
    emit_tls_session_teardown_for_handle(ctx, 0);
    let legacy_filter_cleanup_done = ctx.next_label("fclose_legacy_filter_cleanup_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #256");                            // transitional filter tables cover descriptors below 256 only
            ctx.emitter.instruction(&format!("b.hs {}", legacy_filter_cleanup_done)); // high descriptors have no table-backed filter state
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 256");                            // transitional filter tables cover descriptors below 256 only
            ctx.emitter.instruction(&format!("jae {}", legacy_filter_cleanup_done)); // high descriptors have no table-backed filter state
        }
    }
    if matches!(ctx.emitter.target.arch, Arch::X86_64) {
        ctx.emitter.instruction("mov rdi, rax");                                // pass the descriptor to the user-filter teardown helper
    }
    abi::emit_call_label(ctx.emitter, "__rt_user_filter_release_fd");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_read_filters");
            ctx.emitter.instruction("strb wzr, [x9, x0]");                      // clear any read filter before the descriptor can be reused
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_write_filters");
            ctx.emitter.instruction("strb wzr, [x9, x0]");                      // clear any write filter before the descriptor can be reused
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_read_filters"); // read-filter table base
            ctx.emitter.instruction("mov BYTE PTR [r9 + rax], 0");              // clear any read filter before the descriptor can be reused
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_write_filters"); // write-filter table base
            ctx.emitter.instruction("mov BYTE PTR [r9 + rax], 0");              // clear any write filter before the descriptor can be reused
        }
    }
    ctx.emitter.label(&legacy_filter_cleanup_done);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.syscall(6);
            ctx.emitter.instruction("cmp x0, #0");                              // test whether close() reported success
            ctx.emitter.instruction(&format!("b.eq {}", success_label));        // branch to the true result when the stream closed cleanly
            ctx.emitter.instruction("mov x0, #0");                              // return false when the stream close failed
            ctx.emitter.instruction(&format!("b {}", done_label));              // skip the success result on the failure path
            ctx.emitter.label(&success_label);
            ctx.emitter.instruction("mov x0, #1");                              // return true when the stream closed successfully
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, rax");                            // pass the stream fd to libc close()
            ctx.emitter.instruction("call close");                              // close the requested stream descriptor
            ctx.emitter.instruction("cmp rax, 0");                              // test whether close() reported success
            ctx.emitter.instruction(&format!("je {}", success_label));          // branch to the true result when the stream closed cleanly
            ctx.emitter.instruction("xor eax, eax");                            // return false when the stream close failed
            ctx.emitter.instruction(&format!("jmp {}", done_label));            // skip the success result on the failure path
            ctx.emitter.label(&success_label);
            ctx.emitter.instruction("mov rax, 1");                              // return true when the stream close succeeded
        }
    }
    ctx.emitter.label(&done_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("b {}", after_dispatch_label));    // skip synthetic close handlers after the native fd path
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("jmp {}", after_dispatch_label));  // skip synthetic close handlers after the native fd path
        }
    }
    ctx.emitter.label(&user_wrapper_label);
    if matches!(ctx.emitter.target.arch, Arch::X86_64) {
        ctx.emitter.instruction("mov rdi, rax");                                // pass the synthetic wrapper descriptor to the close helper
    }
    abi::emit_call_label(ctx.emitter, "__rt_user_wrapper_fclose");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("b {}", after_dispatch_label));    // skip phar finalization after wrapper close dispatch
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("jmp {}", after_dispatch_label));  // skip phar finalization after wrapper close dispatch
        }
    }
    ctx.emitter.label(&phar_label);
    if matches!(ctx.emitter.target.arch, Arch::X86_64) {
        ctx.emitter.instruction("mov rdi, rax");                                // pass the PHAR write descriptor to the finalizer
    }
    abi::emit_call_label(ctx.emitter, "__rt_phar_write_finalize");
    ctx.emitter.label(&after_dispatch_label);
    finish_stream_close(ctx);
    store_if_result(ctx, inst)
}

/// Lowers `fread(stream, length)` using the shared runtime file-read helper.
pub(crate) fn lower_fread(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "fread", 2)?;
    let stream = expect_operand(inst, 0)?;
    let length = expect_operand(inst, 1)?;
    load_open_stream_handle_to_result(ctx, stream, "fread")?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    require_int(ctx.load_value_to_result(length)?.codegen_repr(), "fread length")?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x1, x0");                              // pass the requested byte count to the fread runtime helper
            abi::emit_pop_reg(ctx.emitter, "x0");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rsi, rax");                            // pass the requested byte count to the fread runtime helper
            abi::emit_pop_reg(ctx.emitter, "rdi");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_fread");
    store_if_result(ctx, inst)
}

/// Lowers `fwrite(stream, data)` and boxes a byte count or PHP `false` on error.
pub(crate) fn lower_fwrite(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "fwrite", 2)?;
    let stream = expect_operand(inst, 0)?;
    let data = expect_operand(inst, 1)?;
    load_open_stream_handle_to_result(ctx, stream, "fwrite")?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg(ctx.emitter, "x0");
            load_string_to_result(ctx, data, "fwrite data")?;
            abi::emit_pop_reg(ctx.emitter, "x0");
            abi::emit_call_label(ctx.emitter, "__rt_fwrite_filtered");
        }
        Arch::X86_64 => {
            abi::emit_push_reg(ctx.emitter, "rax");
            load_string_to_result(ctx, data, "fwrite data")?;
            abi::emit_pop_reg(ctx.emitter, "rdi");
            ctx.emitter.instruction("mov rsi, rax");                            // pass the string pointer to the runtime fwrite helper
            abi::emit_call_label(ctx.emitter, "__rt_fwrite_filtered");
        }
    }
    box_negative_int_or_false_result(ctx, "fwrite");
    store_if_result(ctx, inst)
}

/// Lowers `fprintf(stream, format, values...)` as `sprintf()` plus stream write.
pub(crate) fn lower_fprintf(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count_between(inst, "fprintf", 2, usize::MAX)?;
    let stream = expect_operand(inst, 0)?;
    let format = expect_operand(inst, 1)?;
    let spec_cats = super::super::strings::sprintf_spec_cats_for_format(ctx, format)?;
    load_open_stream_handle_to_result(ctx, stream, "fprintf")?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    for index in (2..inst.operands.len()).rev() {
        let value = expect_operand(inst, index)?;
        let spec_cat = spec_cats.get(index - 2).copied();
        super::super::strings::pack_sprintf_like_arg(ctx, value, spec_cat, "fprintf")?;
    }
    load_string_to_result(ctx, format, "fprintf format")?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("mov x0, #{}", inst.operands.len() - 2)); // pass the number of packed fprintf operands
        }
        Arch::X86_64 => {
            abi::emit_load_int_immediate(ctx.emitter, "rdi", (inst.operands.len() - 2) as i64);
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_sprintf");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_pop_reg(ctx.emitter, "x0");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rsi, rax");                            // pass the formatted string pointer to fwrite
            abi::emit_pop_reg(ctx.emitter, "rdi");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_fwrite_filtered");
    store_if_result(ctx, inst)
}

/// Lowers `vfprintf(stream, format, values)` through `__rt_vsprintf` then fwrite.
pub(crate) fn lower_vfprintf(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "vfprintf", 3)?;
    let stream = expect_operand(inst, 0)?;
    let format = expect_operand(inst, 1)?;
    let values = expect_operand(inst, 2)?;
    load_open_stream_handle_to_result(ctx, stream, "vfprintf")?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("sub sp, sp, #32");                         // reserve fd and format scratch storage
            ctx.emitter.instruction("str x0, [sp, #0]");                        // save the descriptor across formatting
            load_string_to_result(ctx, format, "vfprintf format")?;
            ctx.emitter.instruction("stp x1, x2, [sp, #8]");                    // save the format pointer and length
            ctx.load_value_to_result(values)?;
            ctx.emitter.instruction("ldp x1, x2, [sp, #8]");                    // restore the format pointer and length
            abi::emit_call_label(ctx.emitter, "__rt_vsprintf");
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // reload the destination descriptor
            abi::emit_call_label(ctx.emitter, "__rt_fwrite_filtered");
            ctx.emitter.instruction("add sp, sp, #32");                         // release vfprintf scratch storage
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("sub rsp, 32");                             // reserve fd and format scratch storage
            ctx.emitter.instruction("mov QWORD PTR [rsp], rax");                // save the descriptor across formatting
            load_string_to_result(ctx, format, "vfprintf format")?;
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rax");            // save the format pointer
            ctx.emitter.instruction("mov QWORD PTR [rsp + 16], rdx");           // save the format byte length
            ctx.load_value_to_result(values)?;
            ctx.emitter.instruction("mov rdi, rax");                            // pass the values array to vsprintf
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 8]");            // restore the format pointer
            ctx.emitter.instruction("mov rdx, QWORD PTR [rsp + 16]");           // restore the format byte length
            abi::emit_call_label(ctx.emitter, "__rt_vsprintf");
            ctx.emitter.instruction("mov rsi, rax");                            // pass the formatted string pointer to fwrite
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp]");                // reload the destination descriptor
            abi::emit_call_label(ctx.emitter, "__rt_fwrite_filtered");
            ctx.emitter.instruction("add rsp, 32");                             // release vfprintf scratch storage
        }
    }
    store_if_result(ctx, inst)
}

/// Lowers `fscanf(stream, format)` through `__rt_fgets` and `__rt_sscanf`.
pub(crate) fn lower_fscanf(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count_between(inst, "fscanf", 2, usize::MAX)?;
    let stream = expect_operand(inst, 0)?;
    let format = expect_operand(inst, 1)?;
    load_open_stream_handle_to_result(ctx, stream, "fscanf")?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_call_label(ctx.emitter, "__rt_fgets");
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
            load_string_to_result(ctx, format, "fscanf format")?;
            ctx.emitter.instruction("mov x3, x1");                              // pass the format pointer as the secondary string argument
            ctx.emitter.instruction("mov x4, x2");                              // pass the format length as the secondary string argument
            abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, rax");                            // pass the opaque stream handle to fgets
            abi::emit_call_label(ctx.emitter, "__rt_fgets");
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
            load_string_to_result(ctx, format, "fscanf format")?;
            ctx.emitter.instruction("mov rdi, rax");                            // pass the format pointer as the secondary string argument
            ctx.emitter.instruction("mov rsi, rdx");                            // pass the format length as the secondary string argument
            abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_sscanf");
    store_if_result(ctx, inst)
}

/// Lowers `fgets(stream)` through the shared line-read runtime helper.
/// php-src's verbatim `ValueError` wording for `fgets()` with a non-positive `$length`.
const FGETS_NON_POSITIVE_LENGTH_MESSAGE: &str =
    "fgets(): Argument #2 ($length) must be greater than 0";

pub(crate) fn lower_fgets(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count_between(inst, "fgets", 1, 2)?;
    let stream = expect_operand(inst, 0)?;
    // PHP's optional `$length` bounds the line at `$length - 1` bytes. Zero means unbounded here,
    // which is what an omitted argument resolves to, so the helper needs no separate flag.
    match inst.operands.get(1).copied() {
        None => {
            load_open_stream_handle_to_result(ctx, stream, "fgets")?;
            match ctx.emitter.target.arch {
                Arch::AArch64 => ctx.emitter.instruction("mov x1, #0"),          // no bound
                Arch::X86_64 => {
                    ctx.emitter.instruction("mov rdi, rax");                     // the opaque stream handle
                    ctx.emitter.instruction("xor esi, esi");                     // no bound
                }
            }
        }
        Some(length) => {
            resolve_int_operand_to_result(ctx, length, "fgets length")?;
            abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
            load_open_stream_handle_to_result(ctx, stream, "fgets")?;
            let bound_reg = match ctx.emitter.target.arch {
                Arch::AArch64 => {
                    abi::emit_pop_reg(ctx.emitter, "x1");                        // the requested bound
                    "x1"
                }
                Arch::X86_64 => {
                    ctx.emitter.instruction("mov rdi, rax");                     // the opaque stream handle
                    abi::emit_pop_reg(ctx.emitter, "rsi");                       // the requested bound
                    "rsi"
                }
            };
            // Zero is what an omitted argument means to the helper, so a caller-supplied zero
            // must never reach it. php-src rejects zero and negatives outright.
            super::super::exceptions::emit_value_error_unless(
                ctx,
                super::super::exceptions::ValueGuard::SignedAtLeast(bound_reg, 1),
                FGETS_NON_POSITIVE_LENGTH_MESSAGE,
            );
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_fgets");
    box_stream_string_or_false_on_empty_result(ctx, "fgets");
    store_if_result(ctx, inst)
}

/// Lowers `fgetc(stream)` and boxes the one-byte string or PHP false result.
pub(crate) fn lower_fgetc(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "fgetc", 1)?;
    let stream = expect_operand(inst, 0)?;
    load_open_stream_handle_to_result(ctx, stream, "fgetc")?;
    if ctx.emitter.target.arch == Arch::X86_64 {
        ctx.emitter.instruction("mov rdi, rax");                                // pass the opaque stream handle to the x86_64 fgetc helper
    }
    abi::emit_call_label(ctx.emitter, "__rt_fgetc");
    box_stream_string_or_false_on_empty_result(ctx, "fgetc");
    store_if_result(ctx, inst)
}

/// Lowers `fgetcsv(stream, length?, separator?, enclosure?, escape?)` through the CSV row
/// runtime helper, passing separator/enclosure/escape as a packed `csv_opts` word.
pub(crate) fn lower_fgetcsv(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count_between(inst, "fgetcsv", 1, 5)?;
    let stream = expect_operand(inst, 0)?;
    let arch = ctx.emitter.target.arch;
    load_open_stream_handle_to_result(ctx, stream, "fgetcsv")?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));            // save the opaque stream handle on stack

    // -- extract first byte of separator / enclosure / escape (or default) --
    let csv_indices: [(usize, u8, &str); 3] = [
        (2, b',', "fgetcsv separator"),
        (3, b'"', "fgetcsv enclosure"),
        (4, b'\\', "fgetcsv escape"),
    ];
    for (idx, default, name) in csv_indices {
        if inst.operands.len() > idx {
            let v = expect_operand(inst, idx)?;
            load_string_to_result(ctx, v, name)?;
            let empty_label = ctx.next_label("csv_empty");
            let done_label = ctx.next_label("csv_done");
            match arch {
                Arch::AArch64 => {
                    ctx.emitter.instruction(&format!("cbz x2, {}", empty_label)); // branch if string is empty
                    ctx.emitter.instruction("ldrb w0, [x1]");                   // load first byte of the CSV delimiter string
                    ctx.emitter.instruction(&format!("b {}", done_label));      // skip the empty-string fallback
                    ctx.emitter.label(&empty_label);
                    ctx.emitter.instruction("mov w0, #0");                      // use zero byte when the string is empty
                    ctx.emitter.label(&done_label);
                }
                Arch::X86_64 => {
                    ctx.emitter.instruction("test rdx, rdx");                   // check string length for the CSV delimiter
                    ctx.emitter.instruction(&format!("jz {}", empty_label));    // branch if string is empty
                    ctx.emitter.instruction("movzx eax, BYTE PTR [rax]");       // load first byte of the CSV delimiter string
                    ctx.emitter.instruction(&format!("jmp {}", done_label));    // skip the empty-string fallback
                    ctx.emitter.label(&empty_label);
                    ctx.emitter.instruction("mov eax, 0");                      // use zero byte when the string is empty
                    ctx.emitter.label(&done_label);
                }
            }
        } else {
            match arch {
                Arch::AArch64 => {
                    ctx.emitter.instruction(&format!("mov w0, #{}", default));  // use default CSV delimiter byte
                }
                Arch::X86_64 => {
                    ctx.emitter.instruction(&format!("mov eax, {}", default));  // use default CSV delimiter byte
                }
            }
        }
        abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));        // save extracted delimiter byte
    }

    // -- pack csv_opts = (esc << 16) | (enc << 8) | sep --
    match arch {
        Arch::AArch64 => {
            abi::emit_pop_reg(ctx.emitter, "x1");                                // pop escape byte
            ctx.emitter.instruction("lsl x1, x1, #16");                         // shift escape to bits 16..23
            abi::emit_pop_reg(ctx.emitter, "x0");                                // pop enclosure byte
            ctx.emitter.instruction("orr x1, x1, x0, lsl #8");                  // include enclosure in csv_opts
            abi::emit_pop_reg(ctx.emitter, "x0");                                // pop separator byte
            ctx.emitter.instruction("orr x1, x1, x0");                          // complete csv_opts in x1
            abi::emit_pop_reg(ctx.emitter, "x0");                                // restore the opaque stream handle into x0
        }
        Arch::X86_64 => {
            abi::emit_pop_reg(ctx.emitter, "rax");                               // pop escape byte
            ctx.emitter.instruction("shl rax, 16");                             // shift escape to bits 16..23
            ctx.emitter.instruction("mov rsi, rax");                            // start accumulating csv_opts
            abi::emit_pop_reg(ctx.emitter, "rax");                               // pop enclosure byte
            ctx.emitter.instruction("shl rax, 8");                              // shift enclosure to bits 8..15
            ctx.emitter.instruction("or rsi, rax");                             // include enclosure in csv_opts
            abi::emit_pop_reg(ctx.emitter, "rax");                               // pop separator byte
            ctx.emitter.instruction("or rsi, rax");                             // complete csv_opts in rsi
            abi::emit_pop_reg(ctx.emitter, "rdi");                               // restore the opaque stream handle into rdi
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_fgetcsv");                           // call the CSV row parser runtime
    store_if_result(ctx, inst)
}

/// Lowers `fputcsv(stream, fields, separator?, enclosure?, escape?, eol?)` for string arrays,
/// passing separator/enclosure/escape as a packed `csv_opts` word and eol as (ptr, len).
pub(crate) fn lower_fputcsv(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count_between(inst, "fputcsv", 2, 6)?;
    let stream = expect_operand(inst, 0)?;
    let fields = expect_operand(inst, 1)?;
    let arch = ctx.emitter.target.arch;

    load_stream_fd_to_result(ctx, stream, "fputcsv")?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));            // save stream fd on stack
    require_string_array(ctx.load_value_to_result(fields)?.codegen_repr(), "fputcsv fields")?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));            // save fields array pointer

    // -- extract first byte of separator / enclosure / escape (or default) --
    let csv_indices: [(usize, u8, &str); 3] = [
        (2, b',', "fputcsv separator"),
        (3, b'"', "fputcsv enclosure"),
        (4, 0, "fputcsv escape"),
    ];
    for (idx, default, name) in csv_indices {
        if inst.operands.len() > idx {
            let v = expect_operand(inst, idx)?;
            load_string_to_result(ctx, v, name)?;
            let empty_label = ctx.next_label("csv_empty");
            let done_label = ctx.next_label("csv_done");
            match arch {
                Arch::AArch64 => {
                    ctx.emitter.instruction(&format!("cbz x2, {}", empty_label)); // branch if string is empty
                    ctx.emitter.instruction("ldrb w0, [x1]");                   // load first byte of the CSV delimiter string
                    ctx.emitter.instruction(&format!("b {}", done_label));      // skip the empty-string fallback
                    ctx.emitter.label(&empty_label);
                    ctx.emitter.instruction("mov w0, #0");                      // use zero byte when the string is empty
                    ctx.emitter.label(&done_label);
                }
                Arch::X86_64 => {
                    ctx.emitter.instruction("test rdx, rdx");                   // check string length for the CSV delimiter
                    ctx.emitter.instruction(&format!("jz {}", empty_label));    // branch if string is empty
                    ctx.emitter.instruction("movzx eax, BYTE PTR [rax]");       // load first byte of the CSV delimiter string
                    ctx.emitter.instruction(&format!("jmp {}", done_label));    // skip the empty-string fallback
                    ctx.emitter.label(&empty_label);
                    ctx.emitter.instruction("mov eax, 0");                      // use zero byte when the string is empty
                    ctx.emitter.label(&done_label);
                }
            }
        } else {
            match arch {
                Arch::AArch64 => {
                    ctx.emitter.instruction(&format!("mov w0, #{}", default));  // use default CSV delimiter byte
                }
                Arch::X86_64 => {
                    ctx.emitter.instruction(&format!("mov eax, {}", default));  // use default CSV delimiter byte
                }
            }
        }
        abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));        // save extracted delimiter byte
    }

    // -- pack csv_opts = (esc << 16) | (enc << 8) | sep --
    match arch {
        Arch::AArch64 => {
            abi::emit_pop_reg(ctx.emitter, "x2");                                // pop escape byte
            ctx.emitter.instruction("lsl x2, x2, #16");                         // shift escape to bits 16..23
            abi::emit_pop_reg(ctx.emitter, "x0");                                // pop enclosure byte
            ctx.emitter.instruction("orr x2, x2, x0, lsl #8");                  // include enclosure in csv_opts
            abi::emit_pop_reg(ctx.emitter, "x0");                                // pop separator byte
            ctx.emitter.instruction("orr x2, x2, x0");                          // complete csv_opts in x2
            abi::emit_push_reg(ctx.emitter, "x2");                              // save packed csv_opts
        }
        Arch::X86_64 => {
            abi::emit_pop_reg(ctx.emitter, "rax");                               // pop escape byte
            ctx.emitter.instruction("shl rax, 16");                             // shift escape to bits 16..23
            ctx.emitter.instruction("mov rdx, rax");                            // start accumulating csv_opts
            abi::emit_pop_reg(ctx.emitter, "rax");                               // pop enclosure byte
            ctx.emitter.instruction("shl rax, 8");                              // shift enclosure to bits 8..15
            ctx.emitter.instruction("or rdx, rax");                             // include enclosure in csv_opts
            abi::emit_pop_reg(ctx.emitter, "rax");                               // pop separator byte
            ctx.emitter.instruction("or rdx, rax");                             // complete csv_opts in rdx
            abi::emit_push_reg(ctx.emitter, "rdx");                             // save packed csv_opts
        }
    }

    // -- push eol (ptr, len) or (0, 0) for default --
    if inst.operands.len() > 5 {
        let eol = expect_operand(inst, 5)?;
        load_string_to_result(ctx, eol, "fputcsv eol")?;
        match arch {
            Arch::AArch64 => {
                abi::emit_push_reg(ctx.emitter, "x1");                          // save eol string pointer
                abi::emit_push_reg(ctx.emitter, "x2");                          // save eol string length
            }
            Arch::X86_64 => {
                abi::emit_push_reg(ctx.emitter, "rax");                         // save eol string pointer
                abi::emit_push_reg(ctx.emitter, "rdx");                         // save eol string length
            }
        }
    } else {
        match arch {
            Arch::AArch64 => {
                ctx.emitter.instruction("mov x0, #0");                          // null eol pointer signals default newline
                abi::emit_push_reg(ctx.emitter, "x0");                          // push eol ptr
                abi::emit_push_reg(ctx.emitter, "x0");                          // push eol len
            }
            Arch::X86_64 => {
                ctx.emitter.instruction("mov rax, 0");                          // null eol pointer signals default newline
                abi::emit_push_reg(ctx.emitter, "rax");                         // push eol ptr
                abi::emit_push_reg(ctx.emitter, "rax");                         // push eol len
            }
        }
    }

    // -- pop all into ABI registers: fd, arr, csv_opts, eol_ptr, eol_len --
    match arch {
        Arch::AArch64 => {
            abi::emit_pop_reg(ctx.emitter, "x4");                                // eol length -> arg5
            abi::emit_pop_reg(ctx.emitter, "x3");                                // eol pointer -> arg4
            abi::emit_pop_reg(ctx.emitter, "x2");                                // csv_opts -> arg3
            abi::emit_pop_reg(ctx.emitter, "x1");                                // fields array -> arg2
            abi::emit_pop_reg(ctx.emitter, "x0");                                // stream fd -> arg1
        }
        Arch::X86_64 => {
            abi::emit_pop_reg(ctx.emitter, "r8");                                // eol length -> arg5
            abi::emit_pop_reg(ctx.emitter, "rcx");                               // eol pointer -> arg4
            abi::emit_pop_reg(ctx.emitter, "rdx");                               // csv_opts -> arg3
            abi::emit_pop_reg(ctx.emitter, "rsi");                               // fields array -> arg2
            abi::emit_pop_reg(ctx.emitter, "rdi");                               // stream fd -> arg1
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_fputcsv");                           // call the CSV row writer runtime
    store_if_result(ctx, inst)
}

/// Lowers `fpassthru(stream)` through the remaining-bytes stream runtime helper.
pub(crate) fn lower_fpassthru(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "fpassthru", 1)?;
    let stream = expect_operand(inst, 0)?;
    load_open_stream_handle_to_result(ctx, stream, "fpassthru")?;
    emit_fpassthru_dispatch(ctx);
    store_if_result(ctx, inst)
}
