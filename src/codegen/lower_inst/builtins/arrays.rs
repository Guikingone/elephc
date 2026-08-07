//! Purpose:
//! Lowers small indexed-array and associative-array builtins for the EIR backend.
//! Delegates aggregate iteration, set operations, and key checks to existing runtime helpers.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::lower_language_construct_call()`.
//!
//! Key details:
//! - Aggregate helpers accept indexed arrays with 8-byte payload slots, and
//!   dispatch to refcount-aware runtime variants when payloads own heap values.
//! - Associative key filters require hash operands because their runtime helpers copy hash entries.

use crate::codegen::platform::Arch;
use crate::codegen::{
    abi, callable_descriptor, callable_dispatch, emit_box_current_owned_value_as_mixed,
    emit_box_current_value_as_mixed,
};
use crate::codegen::{CodegenIrError, Result};
use crate::codegen_support::runtime::HashMapResultKind;
use crate::codegen_support::DeferredCallbackWrapper;
use crate::ir::{BlockId, Immediate, Instruction, LocalSlotId, Op, ValueDef, ValueId};
use crate::names::{function_symbol, method_symbol, php_symbol_key, static_method_symbol};
use crate::types::{array_key_type_from_value_type, PhpType};

use super::super::super::context::FunctionContext;
use super::super::callables::runtime_string_descriptor_cases;
use super::super::{expect_operand, resolve_int_operand_to_result, store_if_result};

mod column;
mod key_exists;
mod keys;
mod search;
mod shift;
mod unshift;
pub(in crate::codegen::lower_inst::builtins) mod values;
mod basic;
mod filter;
mod map_dispatch;
mod map_results;
mod reduce_sets;
mod misc_dispatch;
mod callback_builtins;
mod sort_dispatch;
mod type_validation;
mod callback_binding;
mod callback_sources;
mod callback_targets;
mod callback_wrapper_frame;
mod callback_wrapper_emit;
mod helper_validation;
mod fill_helpers;
mod slice_splice;
mod pop_search;
mod in_array_cases;
mod in_array_coercions;
mod in_array_strings;

use map_dispatch::*;
use map_results::*;
use misc_dispatch::*;
use sort_dispatch::*;
use type_validation::*;
use callback_binding::*;
use callback_sources::*;
use callback_targets::*;
use callback_wrapper_frame::*;
use callback_wrapper_emit::*;
use helper_validation::*;
use fill_helpers::*;
use slice_splice::*;
use pop_search::*;
use in_array_cases::*;
use in_array_coercions::*;
use in_array_strings::*;

pub(crate) use basic::{
    lower_call_user_func_builtin_escape, lower_array_sum, lower_array_product, lower_array_push,
    lower_array_chunk, lower_array_pad, lower_array_fill, lower_array_fill_keys,
    lower_array_combine, lower_array_column, lower_array_flip, lower_array_reverse,
    lower_array_unique,
};
pub(crate) use filter::{
    lower_array_filter,
};
pub(crate) use map_dispatch::{
    lower_array_map,
};
pub(crate) use reduce_sets::{
    lower_array_reduce, lower_array_walk, lower_array_merge, lower_array_diff,
    lower_array_intersect, lower_array_diff_key, lower_array_intersect_key, lower_array_slice,
    lower_array_splice,
};
pub(crate) use misc_dispatch::{
    lower_array_values, lower_array_keys, lower_array_rand, lower_range,
    lower_array_pop, lower_array_shift, lower_array_unshift, lower_sort,
    lower_rsort, lower_asort, lower_arsort, lower_ksort,
    lower_krsort, lower_natsort, lower_natcasesort, lower_shuffle,
    lower_usort, lower_uksort, lower_uasort, lower_array_key_exists,
    lower_array_is_list, lower_array_key_first, lower_array_key_last, lower_array_replace,
    lower_array_replace_recursive, lower_array_diff_assoc, lower_array_intersect_assoc, lower_array_merge_recursive,
};
pub(crate) use callback_builtins::{
    lower_array_find, lower_array_any, lower_array_all, lower_array_walk_recursive,
    lower_array_udiff, lower_array_uintersect, lower_array_multisort, lower_array_search,
    lower_in_array,
};
pub(super) use in_array_cases::InArrayMode;
