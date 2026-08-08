//! Purpose:
//! Lowers object metadata opcodes for the Phase 04 EIR backend.
//! Supports simple object allocation, declared property access, and named or dynamic `instanceof` checks.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()`.
//!
//! Key details:
//! - Object payload layout must match the runtime helpers:
//!   heap kind word before payload, class id at payload offset 0, then 16 bytes
//!   per declared property slot plus an optional dynamic-property hash pointer.
//! - Reference properties store a pointer to a local or heap ref-cell in the
//!   property slot, while normal declared properties store values directly.
//! - This slice intentionally rejects interface method entries that need missing
//!   EIR symbols and non-literal default property expressions until their runtime
//!   paths land.

use std::collections::HashSet;

use crate::codegen::platform::Arch;
use crate::codegen::UNINITIALIZED_TYPED_PROPERTY_SENTINEL;
use crate::codegen_support::sentinels::THROWABLE_CREATION_LINE_OFFSET;
use crate::codegen::{
    abi, callable_descriptor, emit_box_current_owned_value_as_mixed,
    emit_box_current_value_as_mixed, runtime_value_tag,
};
use crate::intrinsics::IntrinsicCall;
use crate::ir::{Immediate, Instruction, LocalSlotId, Op, ValueDef, ValueId};
use crate::names::{method_symbol, php_symbol_key};
use crate::parser::ast::Visibility;
use crate::types::{ClassInfo, InterfaceInfo, PhpType};

use super::super::context::FunctionContext;
use super::{
    builtins, callables, cast_loaded_mixed_pointer_to_result, direct_call_stack_pad_bytes,
    expect_data,
    coerce_loaded_value_to_tagged_scalar, emit_instance_method_descriptor_entry_wrapper,
    emit_loaded_assoc_array_to_mixed,
    emit_loaded_indexed_array_to_mixed, emit_mixed_string_for_persistent_store,
    emit_ref_arg_writebacks, expect_operand, iterators, load_value_to_first_int_arg,
    materialize_method_call_args_with_receiver_reg_and_refs, resolve_method_call_target,
    emit_runtime_callable_invoker_inline, property_values, store_if_result,
    store_method_call_result,
};
use crate::codegen::fibers;
use crate::codegen::literal_defaults::{
    emit_array_literal_default_to_result, emit_assoc_array_literal_default_to_result,
    emit_boxed_bool_literal_to_result, emit_boxed_float_literal_to_result,
    emit_boxed_int_literal_to_result, emit_boxed_null_literal_to_result,
    emit_boxed_string_literal_default_to_result, emit_empty_assoc_array_literal_to_result,
    emit_string_literal_default_to_result, emit_tagged_null_literal_to_result,
    literal_default_value, LiteralDefaultValue,
};
use crate::codegen::{CodegenIrError, Result};

mod reflection;

const RUNTIME_NULL_SENTINEL: i64 = 0x7fff_ffff_ffff_fffe;
const ITERATOR_ITERATOR_DOWNCAST_MESSAGE: &str =
    "Class to downcast to not found or not base class or does not implement Traversable";

/// Resolved declared-property storage metadata for a known object receiver.
struct PropertySlot {
    class_name: String,
    property: String,
    php_type: PhpType,
    offset: usize,
    is_declared: bool,
    is_packed: bool,
    is_reference: bool,
}

/// Declared-property candidate reachable from a `Mixed` object receiver.
struct MixedPropertyCandidate {
    class_id: u64,
    slot: PropertySlot,
}

/// Resolved object property default metadata for fixed-offset initialization.
struct PropertyDefault {
    offset: usize,
    value: LiteralDefaultValue,
    /// `true` when the slot holds a ref-cell pointer (an object-owned reference property);
    /// the default is written THROUGH the cell instead of directly into the slot.
    is_reference: bool,
}

/// Concrete class that a dynamic factory can instantiate in this EIR module.
struct DynamicNewCandidate {
    class_name: String,
    class_id: u64,
    property_count: usize,
    allow_dynamic_properties: bool,
    uninitialized_marker_offsets: Vec<usize>,
    owned_reference_property_offsets: Vec<usize>,
    property_defaults: Vec<PropertyDefault>,
    constructor_impl: Option<ConstructorCallTarget>,
}

/// Constructor metadata needed after object allocation has produced `$this`.
struct ConstructorCallTarget {
    impl_class: String,
    param_types: Vec<PhpType>,
    ref_params: Vec<bool>,
    sig: crate::types::FunctionSig,
}


mod fixed_new;
mod clone_and_spl;
mod iterator_iterator;
mod throwable_new;
mod fiber_dynamic_entry;
mod dynamic_mixed_candidates;
mod dynamic_factory;
mod dynamic_pdo;
mod property_defaults;
mod known_property_reads;
mod mixed_property_reads;
mod dynamic_property_read_entry;
mod dynamic_property_read_resolution;
mod runtime_property_writes;
mod named_property_writes;
mod instanceof_entry;
mod allocation_clone;
mod interface_layout;
mod property_resolution;
mod property_compatibility;
mod property_loads;
mod property_stores;
mod property_store_values;
mod typed_property_guards;
mod instanceof_helpers;

#[allow(unused_imports)]
use fixed_new::*;
#[allow(unused_imports)]
use clone_and_spl::*;
#[allow(unused_imports)]
use iterator_iterator::*;
#[allow(unused_imports)]
use throwable_new::*;
#[allow(unused_imports)]
use fiber_dynamic_entry::*;
#[allow(unused_imports)]
use dynamic_mixed_candidates::*;
#[allow(unused_imports)]
use dynamic_factory::*;
#[allow(unused_imports)]
pub(in crate::codegen::lower_inst) use dynamic_pdo::*;
#[allow(unused_imports)]
use property_defaults::*;
#[allow(unused_imports)]
use known_property_reads::*;
#[allow(unused_imports)]
use mixed_property_reads::*;
#[allow(unused_imports)]
use dynamic_property_read_entry::*;
#[allow(unused_imports)]
use dynamic_property_read_resolution::*;
#[allow(unused_imports)]
use runtime_property_writes::*;
#[allow(unused_imports)]
use named_property_writes::*;
#[allow(unused_imports)]
use instanceof_entry::*;
#[allow(unused_imports)]
use allocation_clone::*;
#[allow(unused_imports)]
use interface_layout::*;
#[allow(unused_imports)]
use property_resolution::*;
#[allow(unused_imports)]
use property_compatibility::*;
#[allow(unused_imports)]
use property_loads::*;
#[allow(unused_imports)]
use property_stores::*;
#[allow(unused_imports)]
use property_store_values::*;
#[allow(unused_imports)]
use typed_property_guards::*;
#[allow(unused_imports)]
use instanceof_helpers::*;

pub(super) use dynamic_property_read_entry::{lower_dynamic_prop_get, lower_nullsafe_prop_get};
pub(super) use fiber_dynamic_entry::{
    lower_dynamic_object_new, lower_dynamic_object_new_mixed,
    lower_dynamic_object_new_without_constructor_mixed,
};
pub(super) use fixed_new::lower_object_new;
pub(super) use instanceof_entry::{lower_instanceof, lower_instanceof_dynamic};
pub(super) use known_property_reads::{
    lower_load_prop_ref_cell, lower_prop_get, lower_prop_initialized,
};
pub(super) use property_resolution::{
    emit_boxed_null, emit_nullable_receiver_object_payload, nullable_object_receiver_class,
    raw_value_php_type,
};
pub(super) use runtime_property_writes::{lower_dynamic_prop_set, lower_prop_set};
pub(super) use clone_and_spl::lower_object_clone_shallow;
