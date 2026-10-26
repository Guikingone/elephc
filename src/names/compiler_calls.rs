//! Purpose:
//! Identifies compiler-generated calls that have no PHP function or method surface.
//!
//! Called from:
//! - Builtin AST builders and the typed EIR call lowerer.
//!
//! Key details:
//! - Identity is carried by `Name`, never recovered from a PHP spelling.
//! - Captured values must remain live until the generated scope retires its snapshot.

/// A compiler-only call target, independent of PHP's namespace and builtin registries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CompilerCall {
    /// Releases and replaces one trusted native user-property slot, returning a pending Throwable.
    NativePropertyReplace,
    /// Restores a native user property selected by its PHP serialized name.
    NativePropertyRestore,
    /// Captures a heap receiver's root cells without retaining destructor-bearing data.
    HeapScopeBegin,
    /// Returns an acquired logical heap element during a mutation callback.
    HeapScopeRead,
    /// Publishes a completed root write.
    HeapScopeRootWritten,
    /// Retires a scope after mutation bookkeeping, including exception paths.
    HeapScopeEnd,
    /// Publishes the native root hole before destructor reentry can observe storage.
    HeapScopeReleaseRoot,
    /// Chains a captured pending exception without transferring its boxed input owner.
    ExceptionChainMixed,
    /// Copies logical bits without retaining a captured payload.
    BorrowedValueSnapshot,
    /// Acquires an owned value while the captured payload remains live.
    #[allow(dead_code)] // Trusted library AST builders also use this operation directly.
    BorrowedValueRead,
    /// Frees scratch without releasing its borrowed payload.
    #[allow(dead_code)] // Trusted library AST builders also use this operation directly.
    BorrowedValueFree,
}

impl CompilerCall {
    /// Returns the PHP storage type of the compiler-generated expression.
    pub(crate) fn result_type(self) -> crate::types::PhpType {
        use crate::types::PhpType;
        match self {
            Self::BorrowedValueSnapshot | Self::HeapScopeBegin => PhpType::Int,
            Self::BorrowedValueRead | Self::HeapScopeRead | Self::NativePropertyReplace | Self::NativePropertyRestore => PhpType::Mixed,
            Self::BorrowedValueFree | Self::HeapScopeRootWritten | Self::HeapScopeEnd | Self::HeapScopeReleaseRoot | Self::ExceptionChainMixed => PhpType::Void,
        }
    }

    /// Returns whether the runtime helper can raise through `__rt_throw_current` instead of
    /// returning a pending Throwable, so owned operands must be visible to the unwinder.
    pub(crate) fn may_unwind(self) -> bool {
        // Restore throws readonly, reference-detachment and receiver errors directly.
        matches!(self, Self::NativePropertyRestore)
    }
}

#[cfg(test)]
mod tests {
    use super::CompilerCall;
    use crate::names::{Name, NameKind};

    /// PHP text, including a fully qualified spelling, cannot grant a compiler capability.
    #[test]
    fn compiler_runtime_name_cannot_be_forged_from_text() {
        let internal = Name::compiler_call(CompilerCall::BorrowedValueRead);
        let source = Name::from_parts(NameKind::FullyQualified, internal.parts.clone());
        assert_eq!(internal.as_str(), source.as_str());
        assert_eq!(internal.compiler_runtime(), Some(CompilerCall::BorrowedValueRead));
        assert_eq!(source.compiler_runtime(), None);
        assert_ne!(internal, source);
        assert_eq!(internal.clone().compiler_runtime(), internal.compiler_runtime());
    }
}
