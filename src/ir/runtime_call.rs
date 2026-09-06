//! Purpose:
//! Defines typed runtime operations referenced by EIR `RuntimeCall` instructions.
//! Keeps PHP builtin names, target registers, ABI placement, and linker symbols out of EIR.
//!
//! Called from:
//! - Backend-neutral builtin lowering through `BuiltinLoweringContext::emit_runtime_call()`.
//! - The EIR validator, printer, and target backend runtime-call dispatcher.
//!
//! Key details:
//! - Each target has one storage-level signature shared by lowering and validation.
//! - Backend code selects the concrete runtime symbol and physical ABI placement.

use crate::ir::{IrHeapKind, IrType};

/// Logical storage signature enforced for a typed runtime operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCallSignature {
    /// One fixed list of storage-level parameters and one fixed result type.
    Fixed {
        /// Operand storage types in source-independent logical order.
        parameters: &'static [IrType],
        /// Storage type produced by the operation.
        result: IrType,
    },
    /// A runtime function whose values carry their polymorphic logical types in EIR.
    Polymorphic {
        /// Minimum accepted operand count after call-argument normalization.
        min_operands: usize,
        /// Maximum accepted operand count, or `None` for a variadic operation.
        max_operands: Option<usize>,
    },
}

/// PHP key-sort operation that requested a guarded nested-array promotion.
///
/// The runtime storage conversion is identical for both directions, but retaining the typed
/// operation lets the backend emit the correct builtin name when a dynamic cell is not an array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrayKeySort {
    /// Ascending key order requested by `ksort()`.
    Ascending,
    /// Descending key order requested by `krsort()`.
    Descending,
}

impl ArrayKeySort {
    /// Returns the PHP builtin name used in runtime diagnostics.
    pub fn php_name(self) -> &'static str {
        match self {
            Self::Ascending => "ksort",
            Self::Descending => "krsort",
        }
    }
}

/// Typed runtime operation selected by backend-neutral EIR lowering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeCallTarget {
    /// Fetches an intermediate array element in write context, installing an
    /// empty child container when the addressed parent slot is missing or null.
    ArrayFetchForWrite,
    /// Executes a runtime include/require through the Magician bridge while
    /// sharing the caller's materialized PHP scope.
    DynamicInclude {
        /// Whether a previously included canonical path should be skipped.
        once: bool,
        /// Whether a missing path is a fatal require rather than a warning-only include.
        required: bool,
        /// Whether strict PHP is active at the physical include call site.
        strict_php: bool,
    },
    /// Promotes an indexed-array payload stored in a boxed Mixed cell to a
    /// Mixed-entry hash and installs the new payload back into that same cell.
    MixedCellPromoteToHash(ArrayKeySort),
    /// Promotes a boxed Mixed cell fetched for write from its parent and marks the returned hash
    /// as already published into that parent-owned cell.
    MixedCellPromoteAttachedToHash(ArrayKeySort),
    /// Creates an independently mutable boxed Mixed cell from one stored
    /// Mixed cell while retaining its tag-4/tag-5 payload ownership.
    MixedCellClone,
    /// Enters or leaves PHP's QUIET property fetch for the operand of `isset`, `empty` or `??`.
    ///
    /// Compiled code reaching an EVAL-OWNED object's property goes through the bridge, and the
    /// bridge raises for an uninitialized typed property unless the mode is on. The interpreter
    /// sets it around its own operands; this is the same door for the compiled side, and the
    /// depth behind it is thread-local so one chain crossing between the two sees one mode.
    ///
    /// PHP fixes the quiet fetch at COMPILE time -- it propagates down property and dim fetch
    /// nodes and never into a call -- so this is emitted only around operands whose whole chain
    /// is a property/dim walk. That keeps "does not cross a call" without a barrier at every
    /// compiled call site.
    EvalQuietPropertyFetch {
        /// Whether this enters the scope (true) or leaves it (false).
        enter: bool,
    },
    /// A one-string-to-one-string transform implemented by the shared runtime.
    UnaryString(UnaryStringRuntime),
    /// A stable runtime function whose target-aware implementation is backend-owned.
    Function(crate::ir::RuntimeFnId),
    /// A source-sensitive runtime function plus the call site's strict-PHP visibility profile.
    ProfiledFunction {
        /// Stable runtime function dispatched by the backend.
        target: crate::ir::RuntimeFnId,
        /// Whether strict PHP is effective at the physical call site.
        strict_php: bool,
    },
}

impl RuntimeCallTarget {
    /// Returns the logical signature shared by EIR validation and backend lowering.
    pub fn signature(self) -> Option<RuntimeCallSignature> {
        match self {
            RuntimeCallTarget::ArrayFetchForWrite => Some(RuntimeCallSignature::Polymorphic {
                min_operands: 2,
                max_operands: Some(2),
            }),
            RuntimeCallTarget::DynamicInclude { .. } => {
                Some(RuntimeCallSignature::Polymorphic {
                    min_operands: 1,
                    max_operands: Some(1),
                })
            }
            RuntimeCallTarget::MixedCellPromoteToHash(_)
            | RuntimeCallTarget::MixedCellPromoteAttachedToHash(_) => {
                Some(RuntimeCallSignature::Fixed {
                    parameters: &[IrType::Heap(IrHeapKind::Mixed)],
                    result: IrType::Heap(IrHeapKind::Hash),
                })
            }
            RuntimeCallTarget::MixedCellClone => Some(RuntimeCallSignature::Fixed {
                parameters: &[IrType::Heap(IrHeapKind::Mixed)],
                result: IrType::Heap(IrHeapKind::Mixed),
            }),
            RuntimeCallTarget::EvalQuietPropertyFetch { .. } => {
                Some(RuntimeCallSignature::Polymorphic {
                    min_operands: 0,
                    max_operands: Some(0),
                })
            }
            RuntimeCallTarget::UnaryString(_) => Some(RuntimeCallSignature::Fixed {
                parameters: &[IrType::Str],
                result: IrType::Str,
            }),
            RuntimeCallTarget::Function(target) => {
                target.descriptor().logical_signature
            }
            RuntimeCallTarget::ProfiledFunction { target, .. } => {
                target.descriptor().logical_signature
            }
        }
    }

    /// Returns the stable backend-neutral spelling used by textual EIR.
    pub fn as_eir(self) -> &'static str {
        match self {
            RuntimeCallTarget::ArrayFetchForWrite => "array.fetch_for_write",
            RuntimeCallTarget::DynamicInclude {
                once: false,
                required: false,
                ..
            } => "include.dynamic",
            RuntimeCallTarget::DynamicInclude {
                once: true,
                required: false,
                ..
            } => "include.dynamic_once",
            RuntimeCallTarget::DynamicInclude {
                once: false,
                required: true,
                ..
            } => "require.dynamic",
            RuntimeCallTarget::DynamicInclude {
                once: true,
                required: true,
                ..
            } => "require.dynamic_once",
            RuntimeCallTarget::MixedCellPromoteToHash(ArrayKeySort::Ascending) => {
                "array.mixed_cell_promote_to_hash_ksort"
            }
            RuntimeCallTarget::MixedCellPromoteToHash(ArrayKeySort::Descending) => {
                "array.mixed_cell_promote_to_hash"
            }
            RuntimeCallTarget::MixedCellPromoteAttachedToHash(ArrayKeySort::Ascending) => {
                "array.mixed_cell_promote_attached_to_hash_ksort"
            }
            RuntimeCallTarget::MixedCellPromoteAttachedToHash(ArrayKeySort::Descending) => {
                "array.mixed_cell_promote_attached_to_hash"
            }
            RuntimeCallTarget::MixedCellClone => "array.mixed_cell_clone",
            RuntimeCallTarget::EvalQuietPropertyFetch { enter: true } => "eval.quiet_fetch_enter",
            RuntimeCallTarget::EvalQuietPropertyFetch { enter: false } => "eval.quiet_fetch_leave",
            RuntimeCallTarget::UnaryString(runtime) => runtime.as_eir(),
            RuntimeCallTarget::Function(target) => target.as_eir(),
            RuntimeCallTarget::ProfiledFunction { target, .. } => target.as_eir(),
        }
    }
}

/// Runtime implementations for PHP string transforms with a `Str -> Str` signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryStringRuntime {
    AddSlashes,
    Base64Encode,
    BinToHex,
    HexToBin,
    HtmlEntityDecode,
    NlToBr,
    QuoteMeta,
    QuotedPrintableEncode,
    RawUrlDecode,
    RawUrlEncode,
    StripSlashes,
    StrReverse,
    StrToLower,
    StrToUpper,
    UrlDecode,
    UrlEncode,
}

impl UnaryStringRuntime {
    /// Returns the stable backend-neutral spelling used by textual EIR and diagnostics.
    pub fn as_eir(self) -> &'static str {
        match self {
            UnaryStringRuntime::AddSlashes => "string.add_slashes",
            UnaryStringRuntime::Base64Encode => "string.base64_encode",
            UnaryStringRuntime::BinToHex => "string.bin_to_hex",
            UnaryStringRuntime::HexToBin => "string.hex_to_bin",
            UnaryStringRuntime::HtmlEntityDecode => "string.html_entity_decode",
            UnaryStringRuntime::NlToBr => "string.nl_to_br",
            UnaryStringRuntime::QuoteMeta => "string.quote_meta",
            UnaryStringRuntime::QuotedPrintableEncode => "string.quoted_printable_encode",
            UnaryStringRuntime::RawUrlDecode => "string.raw_url_decode",
            UnaryStringRuntime::RawUrlEncode => "string.raw_url_encode",
            UnaryStringRuntime::StripSlashes => "string.strip_slashes",
            UnaryStringRuntime::StrReverse => "string.reverse",
            UnaryStringRuntime::StrToLower => "string.to_lower",
            UnaryStringRuntime::StrToUpper => "string.to_upper",
            UnaryStringRuntime::UrlDecode => "string.url_decode",
            UnaryStringRuntime::UrlEncode => "string.url_encode",
        }
    }
}
