//! Purpose:
//! Registers PHP `get_mangled_object_vars()` through the typed Core EIR surface.
//!
//! Called from:
//! - `crate::builtins::system` during builtin inventory collection.
//!
//! Key details:
//! - Projection keeps PHP visibility-mangled keys and ignores the caller's lexical visibility.

use crate::builtins::semantics::{
    BuiltinLoweringContext, BuiltinLoweringError, BuiltinResultType, BuiltinSemanticInput,
    BuiltinSemantics, LoweredBuiltinValue, NormalizedBuiltinCall,
};
use crate::ir::CoreBuiltinOp;
use crate::types::PhpType;

builtin! {
    contract: "get_mangled_object_vars",
    check: super::core_support::check_string_mixed_hash,
    semantics: semantics(),
}

/// Keeps the result layout authoritative even in injected bodies with no checked expression.
const fn semantics() -> BuiltinSemantics {
    let mut semantics = super::core_support::core_builtin_semantics(
        CoreBuiltinOp::GetMangledObjectVars, lower,
    );
    semantics.result_type = BuiltinResultType::Shared(result_type);
    semantics
}

/// Mangled object properties always use a string-keyed map of boxed values.
fn result_type(_input: &BuiltinSemanticInput<'_>) -> PhpType {
    PhpType::AssocArray { key: Box::new(PhpType::Str), value: Box::new(PhpType::Mixed) }
}

/// Emits the typed Core operation through the shared builtin EIR boundary.
fn lower(
    ctx: &mut dyn BuiltinLoweringContext,
    call: &NormalizedBuiltinCall<'_>,
) -> Result<LoweredBuiltinValue, BuiltinLoweringError> {
    super::core_support::lower_core_builtin(ctx, call, CoreBuiltinOp::GetMangledObjectVars)
}
