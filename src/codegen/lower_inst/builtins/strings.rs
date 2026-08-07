//! Purpose:
//! Facade for EIR lowering of PHP string, hashing, compression, formatting, and network builtins.
//! Focused modules own each builtin family while sharing target-aware coercion and ABI helpers.
//!
//! Called from:
//! - `crate::codegen::lower_inst::runtime_functions` dispatch groups.
//! - Sibling IO builtin lowerers for shared hash and printf argument materialization.
//!
//! Key details:
//! - Runtime helpers retain ownership of returned string storage.
//! - Every backend path handles both AArch64 and x86_64 through the shared ABI layer.

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen::{CodegenIrError, Result};
use crate::ir::{Immediate, Instruction, Op, ValueDef, ValueId};
use crate::types::PhpType;

use super::super::super::context::FunctionContext;
use super::super::predicates;
use super::{
    ensure_arg_count, ensure_arg_count_between, expect_operand, io,
    load_value_to_first_int_arg, store_if_result,
};

mod common;
mod compression;
mod hash;
mod network;
mod parse_url;
mod printf;
mod replace_wrap;
mod scalar;
mod search;
mod simple;
mod split;

#[allow(unused_imports)]
use common::*;
#[allow(unused_imports)]
use compression::*;
#[allow(unused_imports)]
use hash::*;
#[allow(unused_imports)]
use network::*;
#[allow(unused_imports)]
use printf::*;
#[allow(unused_imports)]
use replace_wrap::*;
#[allow(unused_imports)]
use scalar::*;
#[allow(unused_imports)]
use search::*;
#[allow(unused_imports)]
use simple::*;
#[allow(unused_imports)]
use split::*;

pub(crate) use compression::{
    lower_gzcompress, lower_gzdeflate, lower_gzinflate, lower_gzuncompress,
};
pub(crate) use hash::{
    lower_crc32, lower_hash, lower_hash_algos, lower_hash_copy, lower_hash_equals,
    lower_hash_final, lower_hash_hmac, lower_hash_init, lower_hash_update, lower_mb_strlen,
    lower_md5, lower_sha1,
};
pub(crate) use network::{lower_inet, lower_ip2long, lower_long2ip};
pub(crate) use parse_url::lower_parse_url;
pub(crate) use printf::{lower_printf, lower_sprintf, lower_vprintf, lower_vsprintf};
pub(crate) use replace_wrap::{lower_str_pad, lower_string_replace, lower_wordwrap};
pub(crate) use scalar::{lower_chr, lower_number_format, lower_ord};
pub(crate) use search::{
    lower_str_contains, lower_str_repeat, lower_string_position, lower_strstr, lower_substr,
    lower_substr_replace,
};
pub(crate) use simple::{
    lower_binary_string_runtime, lower_grapheme_strrev, lower_html_escape, lower_lcfirst,
    lower_trim_like, lower_ucfirst, lower_unary_string_runtime,
};
pub(crate) use split::{lower_explode, lower_implode, lower_sscanf, lower_str_split};

#[allow(unused_imports)]
pub(super) use common::{
    load_string_arg_to_regs, load_value_as_string_to_regs, materialize_truthy_flag,
};
#[allow(unused_imports)]
pub(super) use printf::{
    pack_sprintf_like_arg, sprintf_spec_cats_for_format, SprintfSpecCat,
};
