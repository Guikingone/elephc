//! Purpose:
//! Lowers filesystem metadata builtins for the EIR backend.
//! Reuses the shared runtime stat helpers instead of duplicating platform logic.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::lower_language_construct_call()`.
//!
//! Key details:
//! - Path operands are already evaluated by EIR and are materialized into the
//!   string result registers expected by the shared runtime helpers.

use crate::codegen::{abi, callable_descriptor, emit_box_current_value_as_mixed, NULL_SENTINEL};
use crate::codegen::platform::Arch;
use crate::codegen::{CodegenIrError, Result};
use crate::ir::{Immediate, Instruction, LocalSlotId, Op, ValueDef, ValueId};
use crate::codegen_support::runtime::resources::layout::{
    CONTEXT_NOTIFIER_OFFSET, CONTEXT_OPTIONS_OFFSET, STREAM_BACKEND_AUX_OFFSET,
    STREAM_READ_FILTER_HEAD_OFFSET, STREAM_WRITE_FILTER_HEAD_OFFSET,
    STREAM_BACKEND_KIND_OFFSET, STREAM_BACKEND_POPEN, STREAM_OWNERSHIP_FLAGS_OFFSET,
    STREAM_STATE_FLAG_IS_URL,
};
use crate::types::PhpType;

use super::super::super::context::FunctionContext;
use super::{expect_operand, load_value_to_first_int_arg, store_if_result};
use super::super::resolve_int_operand_to_result;

const STREAM_METADATA_SLOT: usize = 14;
const STREAM_WRAPPER_UNLINK_SLOT: usize = 15;
const STREAM_WRAPPER_MKDIR_SLOT: usize = 17;
const STREAM_WRAPPER_RMDIR_SLOT: usize = 18;
const STREAM_META_TOUCH: usize = 1;
const STREAM_META_OWNER_NAME: usize = 2;
const STREAM_META_OWNER: usize = 3;
const STREAM_META_GROUP_NAME: usize = 4;
const STREAM_META_GROUP: usize = 5;
const STREAM_META_ACCESS: usize = 6;
const STREAM_OPTION_BLOCKING: usize = 1;
const STREAM_OPTION_READ_TIMEOUT: usize = 4;
const TOUCH_ATIME_NOW: u8 = 1;
const TOUCH_MTIME_NOW: u8 = 2;
const TOUCH_BOTH_NOW: u8 = TOUCH_ATIME_NOW | TOUCH_MTIME_NOW;

mod phar_read;
mod fopen_core;
mod fopen_data;
mod fopen_network;
mod fopen_phar;
mod stream_context;
mod stream_copy_queries;
mod stream_filters;
mod stream_buckets;
mod stream_options;
mod stream_sockets;
mod stream_file_ops;
mod stream_file_dispatch;
mod host_directory_process;
mod phar_write;
mod phar_metadata;
mod filesystem_ops;
mod filesystem_principals;
mod touch_pathinfo;
mod stat_ops;
mod wrapper_dispatch;
mod path_call_helpers;
mod stream_dispatch_helpers;
mod filter_helpers;
mod stream_bucket_arch;
mod close_crypto_arch;
mod stream_read_helpers;
mod context_result_helpers;
mod resource_handles;
mod seek_hash_arch;
mod boxing_helpers;
mod string_validation;
#[cfg(test)]
mod resource_tests;

use phar_read::*;
use fopen_data::*;
use fopen_network::*;
use fopen_phar::*;
use stream_filters::*;
use stream_file_dispatch::*;
use phar_write::*;
use filesystem_principals::*;
use touch_pathinfo::*;
use wrapper_dispatch::*;
use path_call_helpers::*;
use stream_dispatch_helpers::*;
use filter_helpers::*;
use stream_bucket_arch::*;
use close_crypto_arch::*;
use stream_read_helpers::*;
use context_result_helpers::*;
use stream_context::*;
use fopen_core::{begin_fopen_context_scope, emit_literal_php_filter_fopen_result,
    emit_request_default_stream_context_handle, finish_fopen_context_scope, LiteralOpenMode};
use resource_handles::*;
use seek_hash_arch::*;
use boxing_helpers::*;
use string_validation::*;

pub(crate) use phar_read::{
    lower_file_get_contents, lower_hash_file, lower_readfile, lower_readline,
};
pub(crate) use fopen_core::{
    lower_fopen,
};
pub(crate) use stream_context::{
    lower_stream_wrapper_register, lower_stream_wrapper_unregister, lower_stream_wrapper_restore, lower_stream_context_create,
    lower_stream_context_get_default, lower_stream_context_set_default, lower_stream_context_set_option, lower_stream_context_set_params,
    lower_stream_context_get_options, lower_stream_context_get_params, lower_stream_get_contents,
};
pub(crate) use stream_copy_queries::{
    lower_stream_copy_to_stream, lower_stream_get_line, lower_stream_get_meta_data, lower_stream_get_wrappers,
    lower_stream_get_transports, lower_stream_get_filters,
};
pub(crate) use stream_filters::{
    lower_stream_filter_register, lower_stream_filter_attach, lower_stream_filter_remove,
};
pub(crate) use stream_buckets::{
    lower_stream_bucket_new, lower_stream_bucket_make_writeable, lower_stream_bucket_append, lower_stream_bucket_prepend,
    lower_stream_is_local, lower_stream_supports_lock,
};
pub(crate) use stream_options::{
    lower_stream_isatty, lower_stream_set_blocking, lower_stream_set_chunk_size, lower_stream_set_buffer,
    lower_stream_set_timeout, lower_stream_select, lower_stream_resolve_include_path,
};
pub(crate) use stream_sockets::{
    lower_stream_socket_server, lower_stream_socket_client, lower_stream_socket_accept, lower_stream_socket_pair,
    lower_stream_socket_get_name, lower_stream_socket_shutdown, lower_stream_socket_enable_crypto, lower_stream_socket_recvfrom,
    lower_stream_socket_sendto,
};
pub(crate) use stream_file_ops::{
    lower_fclose, lower_fread, lower_fwrite, lower_fprintf,
    lower_vfprintf, lower_fscanf, lower_fgets, lower_fgetc,
    lower_fgetcsv, lower_fputcsv, lower_str_getcsv, lower_fpassthru,
};
pub(crate) use stream_file_dispatch::{
    lower_feof, lower_ftell, lower_fseek, lower_rewind,
    lower_ftruncate, lower_fsync, lower_fflush, lower_fdatasync,
    lower_flock,
};
pub(crate) use host_directory_process::{
    lower_disk_free_space, lower_disk_total_space, lower_gethostname, lower_gethostbyname,
    lower_gethostbyaddr, lower_getprotobyname, lower_getprotobynumber, lower_getservbyname,
    lower_getservbyport, lower_opendir, lower_readdir, lower_closedir,
    lower_rewinddir, lower_popen, lower_pclose, lower_fsockopen,
    lower_file, lower_realpath, lower_realpath_cache_get, lower_realpath_cache_size,
};
pub(crate) use phar_write::{
    lower_file_put_contents, lower_elephc_phar_set_compression, lower_elephc_phar_get_metadata, lower_elephc_phar_get_stub,
    lower_elephc_phar_set_metadata, lower_elephc_phar_set_stub,
};
pub(crate) use phar_metadata::{
    lower_elephc_phar_get_file_metadata, lower_elephc_phar_set_file_metadata, lower_elephc_phar_gzip_archive, lower_elephc_phar_bzip2_archive,
    lower_elephc_phar_decompress_archive, lower_elephc_phar_sign_openssl, lower_elephc_phar_sign_hash, lower_elephc_phar_set_zip_password,
    lower_elephc_phar_get_signature_hash, lower_elephc_phar_get_signature_type, lower_elephc_phar_list_entries,
};
pub(crate) use filesystem_ops::{
    lower_file_exists, lower_unlink, lower_mkdir, lower_rmdir,
    lower_chdir, lower_copy, lower_rename, lower_tempnam,
    lower_scandir, lower_glob, lower_chmod, lower_chown,
    lower_chgrp, lower_lchown, lower_lchgrp, lower_umask,
    lower_touch, lower_basename, lower_dirname, lower_fnmatch,
    lower_pathinfo,
};
pub(crate) use stat_ops::{
    lower_getcwd, lower_sys_get_temp_dir, lower_tmpfile, lower_filesize,
    lower_filemtime, lower_linkinfo, lower_symlink, lower_link,
    lower_readlink, lower_fileatime, lower_filectime, lower_fileperms,
    lower_fileowner, lower_filegroup, lower_fileinode, lower_filetype,
    lower_stat, lower_lstat, lower_fstat, lower_clearstatcache,
    lower_is_file, lower_is_dir, lower_is_readable, lower_is_writable,
    lower_is_writeable, lower_is_executable, lower_is_link,
};
pub(super) use boxing_helpers::box_owned_string_or_false_result;
pub(super) use resource_handles::{
    load_resource_payload_to_result, load_stream_fd_to_result, load_stream_handle_to_result,
};
pub(super) use string_validation::load_string_to_result;

/// Emits a literal `file_get_contents("phar://...")` payload through compile-time PHAR extraction.
///
/// The extracted bytes live in read-only `.data`, so a following `$offset`/`$length` window — which
/// trims its input in place and frees a failed read — would move and free a rodata pointer.
/// `persist` therefore copies the entry into an owned heap string before the window runs.
/// Emits a literal `php://filter/…` read: open, read to the end, close.
///
/// `fopen()` on the same URI already worked — it parses the URL, opens the wrapped resource
/// and stamps the filter chain — so this reuses that opener rather than teaching the
/// filesystem helper about filters. The bytes are taken into owned storage BEFORE the close,
/// because the read answers with the stream's own buffer and closing it takes that away.
/// Reads a literal URI whose scheme belongs to no built-in wrapper through `fopen()`.
///
/// php-src has no separate reader here: `file_get_contents` is `php_stream_open_wrapper` followed
/// by `_php_stream_copy_to_mem`, so every scheme the opener knows is readable by definition. The
/// hand-rolled scheme ladder underneath knows `http`, `https` and `ftp` and then falls back to a
/// filename, which is why a registered user wrapper answered `Failed to open stream` here while
/// `fopen()` on the same URI worked. The opener already scans the wrapper registry and reports an
/// unknown scheme the way php does, so delegating covers both outcomes.
fn emit_literal_wrapper_file_get_contents_bytes(
    ctx: &mut FunctionContext<'_>,
    path: &str,
) -> Result<()> {
    fopen_core::emit_literal_fopen_result(ctx, LiteralOpenMode::ReadOnly, path)?;
    emit_open_read_close_tail(ctx, "fgc_wrapper")
}

fn emit_literal_php_filter_file_get_contents_bytes(
    ctx: &mut FunctionContext<'_>,
    path: &str,
) -> Result<()> {
    emit_literal_php_filter_fopen_result(ctx, LiteralOpenMode::ReadOnly, path)?;
    emit_open_read_close_tail(ctx, "fgc_filter")
}

/// Turns a boxed `fopen()` result already in the result register into the bytes it holds.
///
/// Reads the whole stream, takes ownership of the bytes before the close reclaims the buffer,
/// closes the stream php opened on the caller's behalf, and leaves the pair in the string result
/// registers. A failed open leaves a null pointer, which the boxer reads as PHP `false`.
fn emit_open_read_close_tail(ctx: &mut FunctionContext<'_>, label_prefix: &str) -> Result<()> {
    let fail = ctx.next_label(&format!("{label_prefix}_failed"));
    let done = ctx.next_label(&format!("{label_prefix}_done"));
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_reserve_temporary_stack(ctx.emitter, 32);
            ctx.emitter.instruction("ldr x9, [x0]");                            // the boxed open result tag
            ctx.emitter.instruction("cmp x9, #9");                              // did it answer a resource?
            ctx.emitter.instruction(&format!("b.ne {}", fail));                 // a failed open reads as PHP false
            ctx.emitter.instruction("ldr x0, [x0, #8]");                        // the opaque stream handle
            ctx.emitter.instruction("str x0, [sp, #0]");                        // keep it for the close
            abi::emit_call_label(ctx.emitter, "__rt_stream_get_contents");      // x1/x2 = the filtered bytes
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");              // own them before the close reclaims the buffer
            ctx.emitter.instruction("str x1, [sp, #8]");
            ctx.emitter.instruction("str x2, [sp, #16]");
            ctx.emitter.instruction("ldr x0, [sp, #0]");
            abi::emit_call_label(ctx.emitter, "__rt_resource_mark_closed");     // php closes the stream it opened for us
            ctx.emitter.instruction("ldr x0, [sp, #0]");
            abi::emit_call_label(ctx.emitter, "__rt_resource_release");
            ctx.emitter.instruction("ldr x1, [sp, #8]");                        // the owned bytes are the result
            ctx.emitter.instruction("ldr x2, [sp, #16]");
            ctx.emitter.instruction(&format!("b {}", done));
            ctx.emitter.label(&fail);
            ctx.emitter.instruction("mov x1, #0");                              // null string pointer asks the boxer for PHP false
            ctx.emitter.instruction("mov x2, #0");
            ctx.emitter.label(&done);
            abi::emit_release_temporary_stack(ctx.emitter, 32);
        }
        Arch::X86_64 => {
            abi::emit_reserve_temporary_stack(ctx.emitter, 32);
            ctx.emitter.instruction("mov r9, QWORD PTR [rax]");                 // the boxed open result tag
            ctx.emitter.instruction("cmp r9, 9");                               // did it answer a resource?
            ctx.emitter.instruction(&format!("jne {}", fail));                  // a failed open reads as PHP false
            ctx.emitter.instruction("mov rax, QWORD PTR [rax + 8]");            // the opaque stream handle
            ctx.emitter.instruction("mov QWORD PTR [rsp + 0], rax");            // keep it for the close
            ctx.emitter.instruction("mov rdi, rax");
            abi::emit_call_label(ctx.emitter, "__rt_stream_get_contents");      // rax/rdx = the filtered bytes
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");              // own them before the close reclaims the buffer
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rax");
            ctx.emitter.instruction("mov QWORD PTR [rsp + 16], rdx");
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");
            abi::emit_call_label(ctx.emitter, "__rt_resource_mark_closed");     // php closes the stream it opened for us
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");
            abi::emit_call_label(ctx.emitter, "__rt_resource_release");
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 8]");            // the owned bytes are the result
            ctx.emitter.instruction("mov rdx, QWORD PTR [rsp + 16]");
            ctx.emitter.instruction(&format!("jmp {}", done));
            ctx.emitter.label(&fail);
            ctx.emitter.instruction("xor eax, eax");                            // null string pointer asks the boxer for PHP false
            ctx.emitter.instruction("xor edx, edx");
            ctx.emitter.label(&done);
            abi::emit_release_temporary_stack(ctx.emitter, 32);
        }
    }
    Ok(())
}

/// Emits the decoded payload of a literal `data://` URI as the read's bytes.
///
/// `fopen("data://…")` already decodes these at compile time; `file_get_contents()` went
/// through the filesystem helper instead and answered false with "No such file or directory",
/// naming a path that was never meant to be one. A malformed URI keeps that answer, which is
/// what php does with it too.
fn emit_literal_data_uri_file_get_contents_bytes(
    ctx: &mut FunctionContext<'_>,
    path: &str,
    persist: bool,
) {
    match decode_data_uri_for_fopen(path) {
        Some(payload) => {
            let (symbol, len) = ctx.data.add_string(&payload);
            match ctx.emitter.target.arch {
                Arch::AArch64 => {
                    abi::emit_symbol_address(ctx.emitter, "x1", &symbol);
                    ctx.emitter.instruction(&format!("mov x2, #{}", len));      // decoded data:// payload byte length
                }
                Arch::X86_64 => {
                    abi::emit_symbol_address(ctx.emitter, "rax", &symbol);
                    ctx.emitter.instruction(&format!("mov rdx, {}", len));      // decoded data:// payload byte length
                }
            }
            if persist {
                abi::emit_call_label(ctx.emitter, "__rt_str_persist");
            }
        }
        None => match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.emitter.instruction("mov x1, #0");                          // null string pointer asks the boxer for PHP false
                ctx.emitter.instruction("mov x2, #0");                          // clear the unused failure length
            }
            Arch::X86_64 => {
                ctx.emitter.instruction("xor eax, eax");                        // null string pointer asks the boxer for PHP false
                ctx.emitter.instruction("xor edx, edx");                        // clear the unused failure length
            }
        },
    }
}

fn emit_literal_phar_file_get_contents_bytes(
    ctx: &mut FunctionContext<'_>,
    path: &str,
    persist: bool,
) {
    match crate::codegen::phar_stream::extract_phar_entry(path) {
        Some(payload) => {
            let (symbol, len) = ctx.data.add_string(&payload);
            match ctx.emitter.target.arch {
                Arch::AArch64 => {
                    abi::emit_symbol_address(ctx.emitter, "x1", &symbol);
                    ctx.emitter.instruction(&format!("mov x2, #{}", len));      // embedded phar entry byte length
                }
                Arch::X86_64 => {
                    abi::emit_symbol_address(ctx.emitter, "rax", &symbol);
                    ctx.emitter.instruction(&format!("mov rdx, {}", len));      // embedded phar entry byte length
                }
            }
            if persist {
                abi::emit_call_label(ctx.emitter, "__rt_str_persist");
            }
        }
        None => match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.emitter.instruction("mov x1, #0");                          // null string pointer asks the boxer for PHP false
                ctx.emitter.instruction("mov x2, #0");                          // clear the unused failure length
            }
            Arch::X86_64 => {
                ctx.emitter.instruction("xor eax, eax");                        // null string pointer asks the boxer for PHP false
                ctx.emitter.instruction("xor edx, edx");                        // clear the unused failure length
            }
        },
    }
}
