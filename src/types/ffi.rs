//! Purpose:
//! Defines FFI type and declaration metadata used by checking and native call lowering.
//! Maps compiler extension types into ABI-visible shapes for extern declarations.
//!
//! Called from:
//! - `crate::types::checker::extern_decl`
//! - `crate::codegen`
//!
//! Key details:
//! - FFI metadata must preserve target-independent type contracts while leaving register/stack layout to ABI helpers.
//! - Every C integer width maps to the PHP type `int`; `ExternAbiType` keeps the declared width
//!   so the call boundary can normalize registers without widening the PHP type system.

use std::collections::HashMap;

use crate::parser::ast::CType;

use super::{PackedClassInfo, PhpType};

/// Register-level shape of one extern parameter or return slot, derived from the declared C type.
///
/// The PHP-visible type of every integer slot is `int`; the width only tells the call boundary
/// how to normalize the register, because both supported ABIs leave the bits above a narrow
/// return value unspecified and expect the caller to extend narrow arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternAbiType {
    /// Integer of `bits` width; `signed` selects sign or zero extension at the boundary.
    Int { bits: u8, signed: bool },
    /// C `double` in a floating-point register.
    Float64,
    /// C `bool`: only the low byte is defined.
    Bool,
    /// `char*` converted to and from a PHP string at the boundary.
    CStr,
    /// Any pointer, typed or opaque.
    Ptr,
    /// C function pointer bound from a PHP callable.
    Callable,
    /// No value (return only).
    Void,
}

impl ExternAbiType {
    /// The 64-bit signed integer shape of a plain `int`.
    pub const INT64: Self = Self::Int { bits: 64, signed: true };
}

/// Maps a parser CType to the ABI shape the call boundary normalizes.
pub fn ctype_to_abi_type(ct: &CType) -> ExternAbiType {
    match ct {
        CType::Int => ExternAbiType::INT64,
        CType::IntWidth(width) => ExternAbiType::Int { bits: width.bits(), signed: width.signed() },
        CType::Float => ExternAbiType::Float64,
        CType::Str => ExternAbiType::CStr,
        CType::Bool => ExternAbiType::Bool,
        CType::Void => ExternAbiType::Void,
        CType::Ptr | CType::TypedPtr(_) => ExternAbiType::Ptr,
        CType::Callable => ExternAbiType::Callable,
    }
}

/// Maps a parser CType to its corresponding PhpType for FFI and extern declaration lowering.
///
/// Used when lowering `extern` declarations to connect the parser's C types (Int, IntWidth,
/// Float, Str, Bool, Void, Ptr, TypedPtr, Callable) to the compiler's internal PhpType
/// representation. Every C integer width is a PHP `int`.
pub fn ctype_to_php_type(ct: &CType) -> PhpType {
    match ct {
        CType::Int | CType::IntWidth(_) => PhpType::Int,
        CType::Float => PhpType::Float,
        CType::Str => PhpType::Str,
        CType::Bool => PhpType::Bool,
        CType::Void => PhpType::Void,
        CType::Ptr => PhpType::Pointer(None),
        CType::TypedPtr(name) => PhpType::Pointer(Some(name.clone())),
        CType::Callable => PhpType::Callable,
    }
}

/// Returns the stack frame size in bytes for a C-facing FFI type.
///
/// All integer, float, bool, pointer, typed pointer, callable, and string (char*) types
/// occupy 8 bytes on the stack. Void occupies 0 bytes. Used during codegen frame layout
/// to determine how callee-saved registers and local variables are sized.
pub fn ctype_stack_size(ct: &CType) -> usize {
    match ct {
        // Explicit widths are rejected on extern class fields for now, so a slot stays one word.
        CType::Int
        | CType::IntWidth(_)
        | CType::Float
        | CType::Bool
        | CType::Ptr
        | CType::TypedPtr(_)
        | CType::Callable => 8,
        CType::Str => 8, // char*
        CType::Void => 0,
    }
}

/// Returns the ABI size for a PhpType, or None if the type has no fixed size.
pub fn packed_type_size(
    ty: &PhpType,
    packed_classes: &HashMap<String, PackedClassInfo>,
) -> Option<usize> {
    match ty {
        PhpType::Int
        | PhpType::Float
        | PhpType::Bool
        | PhpType::False
        | PhpType::Pointer(_)
        | PhpType::Resource(_) => Some(8),
        PhpType::Packed(name) => packed_classes.get(name).map(|info| info.total_size),
        _ => None,
    }
}
