//! Purpose:
//! File reads, PHAR bridge publication, hashing, and readline lowering.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::io`.
//!
//! Key details:
//! - Preserves target-aware ABI handling, runtime calls, and result ownership.

use super::*;

/// Lowers `file_get_contents(path)` and boxes the runtime string-or-false result.
pub(crate) fn lower_file_get_contents(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count_between(inst, "file_get_contents", 1, 5)?;
    let path = expect_operand(inst, 0)?;
    let path_literal = optional_const_string_operand(ctx, path)?;
    if let Some(path_literal) = path_literal.as_deref() {
        if path_literal.starts_with("phar://") {
            return lower_literal_phar_file_get_contents(ctx, inst, path_literal);
        }
        if path_literal == "php://input" {
            // file_get_contents('php://input'): under --web `__rt_php_input` copies
            // the captured request body into an owned string; in a non-web build it
            // returns a null pointer so the result boxes to PHP false.
            abi::emit_call_label(ctx.emitter, "__rt_php_input");
            box_owned_string_or_false_result(ctx, "fgc");
            return store_if_result(ctx, inst);
        }
    }
    if path_literal.is_none() {
        publish_dynamic_phar_function_pointers(ctx);
    }
    load_string_to_result(ctx, path, "file_get_contents filename")?;
    abi::emit_call_label(ctx.emitter, "__rt_file_get_contents_maybe_url");
    box_owned_string_or_false_result(ctx, "fgc");
    store_if_result(ctx, inst)
}

/// Publishes bridge/decompressor entry points into runtime slots used by
/// dynamic `phar://` reads.
pub(super) fn publish_dynamic_phar_function_pointers(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[
        ("elephc_phar_extract_url", "_elephc_phar_extract_url_fn"),
        ("inflateInit2_", "_phar_zlib_inflate_init2_fn"),
        ("inflate", "_phar_zlib_inflate_fn"),
        ("inflateEnd", "_phar_zlib_inflate_end_fn"),
        ("BZ2_bzBuffToBuffDecompress", "_phar_bz2_decompress_fn"),
    ];
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            for (c_name, slot) in ENTRIES {
                let extern_sym = ctx.emitter.target.extern_symbol(c_name);
                abi::emit_extern_symbol_address(ctx.emitter, "x9", &extern_sym);
                abi::emit_symbol_address(ctx.emitter, "x10", slot);
                ctx.emitter.instruction("str x9, [x10]");                       // publish the decompressor entry into its runtime slot
            }
        }
        Arch::X86_64 => {
            for (c_name, slot) in ENTRIES {
                let extern_sym = ctx.emitter.target.extern_symbol(c_name);
                abi::emit_extern_symbol_address(ctx.emitter, "r9", &extern_sym);
                abi::emit_store_reg_to_symbol(ctx.emitter, "r9", slot, 0);     // publish the decompressor entry into its runtime slot
            }
        }
    }
}

/// Publishes a list of elephc-phar bridge entry points into runtime slots.
pub(super) fn publish_phar_bridge_entries(ctx: &mut FunctionContext<'_>, entries: &[(&str, &str)]) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            for (c_name, slot) in entries {
                let extern_sym = ctx.emitter.target.extern_symbol(c_name);
                abi::emit_extern_symbol_address(ctx.emitter, "x9", &extern_sym);
                abi::emit_symbol_address(ctx.emitter, "x10", slot);
                ctx.emitter.instruction("str x9, [x10]");                       // publish the PHAR bridge entry into its runtime slot
            }
        }
        Arch::X86_64 => {
            for (c_name, slot) in entries {
                let extern_sym = ctx.emitter.target.extern_symbol(c_name);
                abi::emit_extern_symbol_address(ctx.emitter, "r9", &extern_sym);
                abi::emit_store_reg_to_symbol(ctx.emitter, "r9", slot, 0);     // publish the PHAR bridge entry into its runtime slot
            }
        }
    }
}

/// Publishes the native PHAR read-modify-write bridge used by write finalization.
pub(super) fn publish_phar_write_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[
        ("elephc_phar_put_entry", "_elephc_phar_put_entry_fn"),
        (
            "elephc_phar_stream_open_entry",
            "_elephc_phar_stream_open_entry_fn",
        ),
        ("elephc_phar_stream_append", "_elephc_phar_stream_append_fn"),
        (
            "elephc_phar_stream_finalize",
            "_elephc_phar_stream_finalize_fn",
        ),
    ];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the native PHAR writer bridge used by runtime-built phar:// URLs.
pub(super) fn publish_dynamic_phar_write_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[
        ("elephc_phar_put_url", "_elephc_phar_put_url_fn"),
        (
            "elephc_phar_stream_open_url",
            "_elephc_phar_stream_open_url_fn",
        ),
        ("elephc_phar_stream_append", "_elephc_phar_stream_append_fn"),
        (
            "elephc_phar_stream_finalize",
            "_elephc_phar_stream_finalize_fn",
        ),
    ];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the native PHAR deletion bridge used by `unlink("phar://...")`.
pub(super) fn publish_phar_delete_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[(
        "elephc_phar_delete_url",
        "_elephc_phar_delete_url_fn",
    )];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the native PHAR compression-control bridge.
pub(super) fn publish_phar_set_compression_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[(
        "elephc_phar_set_compression",
        "_elephc_phar_set_compression_fn",
    )];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the archive-entry listing bridge used by PHAR OOP constructors.
pub(super) fn publish_phar_list_entries_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[(
        "elephc_phar_list_entries",
        "_elephc_phar_list_entries_fn",
    )];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the archive global-metadata read bridge.
pub(super) fn publish_phar_get_metadata_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] =
        &[("elephc_phar_get_metadata", "_elephc_phar_get_metadata_fn")];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the archive global-metadata write bridge.
pub(super) fn publish_phar_set_metadata_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] =
        &[("elephc_phar_set_metadata", "_elephc_phar_set_metadata_fn")];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the archive stub read bridge.
pub(super) fn publish_phar_get_stub_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[("elephc_phar_get_stub", "_elephc_phar_get_stub_fn")];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the archive stub write bridge.
pub(super) fn publish_phar_set_stub_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[("elephc_phar_set_stub", "_elephc_phar_set_stub_fn")];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the per-file metadata read bridge.
pub(super) fn publish_phar_get_file_metadata_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[(
        "elephc_phar_get_file_metadata",
        "_elephc_phar_get_file_metadata_fn",
    )];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the per-file metadata write bridge.
pub(super) fn publish_phar_set_file_metadata_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[(
        "elephc_phar_set_file_metadata",
        "_elephc_phar_set_file_metadata_fn",
    )];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the whole-archive gzip compression bridge.
pub(super) fn publish_phar_gzip_archive_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] =
        &[("elephc_phar_gzip_archive", "_elephc_phar_gzip_archive_fn")];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the whole-archive bzip2 compression bridge.
pub(super) fn publish_phar_bzip2_archive_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] =
        &[("elephc_phar_bzip2_archive", "_elephc_phar_bzip2_archive_fn")];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the whole-archive decompression bridge.
pub(super) fn publish_phar_decompress_archive_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[(
        "elephc_phar_decompress_archive",
        "_elephc_phar_decompress_archive_fn",
    )];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the OpenSSL (RSA-SHA1) signing bridge.
pub(super) fn publish_phar_sign_openssl_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] =
        &[("elephc_phar_sign_openssl", "_elephc_phar_sign_openssl_fn")];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the hash-based signing bridge.
pub(super) fn publish_phar_sign_hash_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[("elephc_phar_sign_hash", "_elephc_phar_sign_hash_fn")];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the ZipCrypto password bridge used to read encrypted ZIP entries.
pub(super) fn publish_phar_set_zip_password_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[(
        "elephc_phar_set_zip_password",
        "_elephc_phar_set_zip_password_fn",
    )];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the signature-hash read bridge.
pub(super) fn publish_phar_get_signature_hash_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[(
        "elephc_phar_get_signature_hash",
        "_elephc_phar_get_signature_hash_fn",
    )];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Publishes the signature-type read bridge.
pub(super) fn publish_phar_get_signature_type_function_pointer(ctx: &mut FunctionContext<'_>) {
    const ENTRIES: &[(&str, &str)] = &[(
        "elephc_phar_get_signature_type",
        "_elephc_phar_get_signature_type_fn",
    )];
    publish_phar_bridge_entries(ctx, ENTRIES);
}

/// Lowers `hash_file(algo, filename, binary?)` by reading bytes then hashing them.
pub(crate) fn lower_hash_file(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count_between(inst, "hash_file", 2, 3)?;
    let fail = ctx.next_label("hash_file_fail");
    let done = ctx.next_label("hash_file_box");
    match ctx.emitter.target.arch {
        Arch::AArch64 => lower_hash_file_aarch64(ctx, inst, &fail, &done)?,
        Arch::X86_64 => lower_hash_file_x86_64(ctx, inst, &fail, &done)?,
    }
    box_owned_string_or_false_result(ctx, "hash_file");
    store_if_result(ctx, inst)
}

/// Lowers `readfile(path)` and boxes the runtime byte-count-or-false result.
pub(crate) fn lower_readfile(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count_between(inst, "readfile", 1, 3)?;
    let path = expect_operand(inst, 0)?;
    load_string_to_result(ctx, path, "readfile")?;
    emit_readfile_wrapper_dispatch(ctx);
    box_readfile_result(ctx);
    store_if_result(ctx, inst)
}

/// Lowers `readline(prompt?)` by optionally writing a prompt and reading stdin.
pub(crate) fn lower_readline(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count_between(inst, "readline", 0, 1)?;
    if inst.operands.len() == 1 {
        let prompt = expect_operand(inst, 0)?;
        load_string_to_result(ctx, prompt, "readline prompt")?;
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.emitter.instruction("bl __rt_vd_write");                    // write x1/x2 through the ob/web-aware stdout sink (register-preserving)
            }
            Arch::X86_64 => {
                ctx.emitter.instruction("mov rsi, rax");                        // pass the prompt pointer as write()'s buffer argument
                ctx.emitter.instruction("mov rdi, 1");                          // pass stdout as the destination fd for the readline prompt
                ctx.emitter.instruction("call write");                          // write the prompt before blocking on stdin
            }
        }
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, #0");                              // pass stdin fd 0 to the shared line-reader helper
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("xor edi, edi");                            // pass stdin fd 0 to the shared line-reader helper
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_fgets");
    store_if_result(ctx, inst)
}

