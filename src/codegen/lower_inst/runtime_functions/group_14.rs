//! Purpose:
//! Dispatches the iconv and `ext/curl` group of typed builtin runtime targets.
//!
//! Called from:
//! - `super::lower()` while lowering typed EIR runtime calls.
//!
//! Key details:
//! - Dispatch is by enum identity, never by PHP function-name strings.
//! - This group owns the iconv and curl extension families.

use crate::codegen::context::FunctionContext;
use crate::codegen::Result;
use crate::ir::{Instruction, RuntimeFnId};

/// Lowers a target owned by bounded dispatch group 14, or returns `None`.
pub(super) fn lower(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    target: RuntimeFnId,
) -> Option<Result<()>> {
    match target {
        RuntimeFnId::Iconv => Some({
            crate::codegen::lower_inst::builtins::iconv::lower_iconv(ctx, inst)
        }),
        RuntimeFnId::IconvGetEncoding => Some({
            crate::codegen::lower_inst::builtins::iconv::lower_iconv_get_encoding(ctx, inst)
        }),
        RuntimeFnId::IconvMimeDecode => Some({
            crate::codegen::lower_inst::builtins::iconv::lower_iconv_mime_decode(ctx, inst)
        }),
        RuntimeFnId::IconvMimeDecodeHeaders => Some({
            crate::codegen::lower_inst::builtins::iconv::lower_iconv_mime_decode_headers(ctx, inst)
        }),
        RuntimeFnId::IconvMimeEncode => Some({
            crate::codegen::lower_inst::builtins::iconv::lower_iconv_mime_encode(ctx, inst)
        }),
        RuntimeFnId::IconvSetEncoding => Some({
            crate::codegen::lower_inst::builtins::iconv::lower_iconv_set_encoding(ctx, inst)
        }),
        RuntimeFnId::IconvStrlen => Some({
            crate::codegen::lower_inst::builtins::iconv::lower_iconv_strlen(ctx, inst)
        }),
        RuntimeFnId::IconvStrpos => Some({
            crate::codegen::lower_inst::builtins::iconv::lower_iconv_strpos(ctx, inst)
        }),
        RuntimeFnId::IconvStrrpos => Some({
            crate::codegen::lower_inst::builtins::iconv::lower_iconv_strrpos(ctx, inst)
        }),
        RuntimeFnId::IconvSubstr => Some({
            crate::codegen::lower_inst::builtins::iconv::lower_iconv_substr(ctx, inst)
        }),
        RuntimeFnId::CurlEasyBody => Some({
            crate::codegen::lower_inst::builtins::curl::lower_curl_easy_body(ctx, inst)
        }),
        RuntimeFnId::CurlEasyErrno => Some({
            crate::codegen::lower_inst::builtins::curl::lower_curl_easy_errno(ctx, inst)
        }),
        RuntimeFnId::CurlEasyError => Some({
            crate::codegen::lower_inst::builtins::curl::lower_curl_easy_error(ctx, inst)
        }),
        RuntimeFnId::CurlEasyGetinfoLong => Some({
            crate::codegen::lower_inst::builtins::curl::lower_curl_easy_getinfo_long(ctx, inst)
        }),
        RuntimeFnId::CurlEasyInit => Some({
            crate::codegen::lower_inst::builtins::curl::lower_curl_easy_init(ctx, inst)
        }),
        RuntimeFnId::CurlEasyPerform => Some({
            crate::codegen::lower_inst::builtins::curl::lower_curl_easy_perform(ctx, inst)
        }),
        RuntimeFnId::CurlEasySetoptLong => Some({
            crate::codegen::lower_inst::builtins::curl::lower_curl_easy_setopt_long(ctx, inst)
        }),
        RuntimeFnId::CurlEasySetoptStr => Some({
            crate::codegen::lower_inst::builtins::curl::lower_curl_easy_setopt_str(ctx, inst)
        }),
        RuntimeFnId::CurlEasySetoptSlist => Some({
            crate::codegen::lower_inst::builtins::curl::lower_curl_easy_setopt_slist(ctx, inst)
        }),
        RuntimeFnId::CurlOptionKind => Some({
            crate::codegen::lower_inst::builtins::curl::lower_curl_option_kind(ctx, inst)
        }),
        RuntimeFnId::CurlSetoptUnsupportedWarning => Some({
            crate::codegen::lower_inst::builtins::curl::lower_curl_setopt_unsupported_warning(
                ctx, inst,
            )
        }),
        RuntimeFnId::CurlVersion => Some({
            crate::codegen::lower_inst::builtins::curl::lower_curl_version(ctx, inst)
        }),
        _ => None,
    }
}
