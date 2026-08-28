//! Purpose:
//! Home of the PHP `openssl_encrypt` builtin and its typed crypto runtime target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through the builtin registry.
//!
//! Key details:
//! - `$tag` is an optional by-reference output and is validated without reading an undefined local.
//! - Dedicated argument lowering promotes tag storage before the target-aware AEAD writeback.

use crate::builtins::semantics::{
    runtime_fn_semantics, with_argument_lowering, BuiltinArgumentLowering, BuiltinSemantics,
};
use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

builtin! {
    contract: "openssl_encrypt",
    check: check,
    lazy_check: true,
    semantics: openssl_encrypt_semantics(),
}

/// Builds encrypt semantics that preserve the by-reference tag target during EIR lowering.
const fn openssl_encrypt_semantics() -> BuiltinSemantics {
    with_argument_lowering(
        runtime_fn_semantics(crate::ir::RuntimeFnId::OpensslEncrypt),
        BuiltinArgumentLowering::OpensslEncrypt,
    )
}

/// Validates the optional by-reference tag target and returns `string|false`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    for (index, arg) in cx.args.iter().enumerate() {
        if index != 5 {
            cx.checker.infer_type(arg, cx.env)?;
        }
    }
    if let Some(tag) = cx.args.get(5) {
        if !matches!(tag.kind, ExprKind::Variable(_)) {
            return Err(CompileError::new(
                tag.span,
                "openssl_encrypt() parameter $tag must be passed a variable",
            ));
        }
    }
    Ok(cx
        .checker
        .normalize_union_type(vec![PhpType::Str, PhpType::False]))
}
