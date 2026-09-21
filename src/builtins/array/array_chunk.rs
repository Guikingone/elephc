//! Purpose:
//! Home of the PHP `array_chunk` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - PHP's signature is `array_chunk(array $array, int $length, bool $preserve_keys = false)`;
//!   both the positional and the `preserve_keys:` named form are accepted.
//! - `preserve_keys` CHANGES THE RESULT SHAPE, so it must be a literal in AOT mode (same rule as
//!   `array_reverse()`'s and `array_slice()`'s flags). With `false` an indexed `Array<T>` chunks
//!   into `Array<Array<T>>`; with `true` each chunk keeps the source integer keys of its own
//!   window, which is `Array<AssocArray { key: Int, value: T }>` because elephc's dense indexed
//!   representation cannot hold a window that does not start at key 0.
//! - A non-array input is rejected. An associative or gradual (`mixed`, or a union carrying an
//!   array) input is accepted ONLY in the two-argument form, where `ir_lower::expr::compat_preludes`
//!   redirects the call to `__elephc_array_chunk_gradual`: the native lowering walks dense indexed
//!   storage and picks its helper from the source element type, so it has nothing to run against
//!   hash storage or a boxed Mixed cell. Those shapes answer `array<mixed>`, which is what the
//!   prelude helper's declared `array` return produces.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::parser::ast::{Expr, ExprKind};
use crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable;
use crate::types::PhpType;

builtin! {
    contract: "array_chunk",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayChunk,
    ),
}

/// Reads a literal `preserve_keys` flag, returning `None` when the argument is not a literal.
///
/// An absent argument reads as a literal `false` so callers can treat "omitted" and "explicit
/// false" identically. Integer literals follow PHP truthiness, matching `array_reverse()`.
fn literal_preserve_keys(flag: Option<&Expr>) -> Option<bool> {
    match flag {
        None => Some(false),
        Some(flag) => match flag.kind {
            ExprKind::BoolLiteral(value) => Some(value),
            ExprKind::IntLiteral(value) => Some(value != 0),
            _ => None,
        },
    }
}

/// Returns the nested chunk-array type for an `array_chunk` call.
///
/// An indexed `Array<elem>` chunks into `Array<Array<elem>>`, or into
/// `Array<AssocArray { key: Int, value: elem }>` when a literal `preserve_keys: true` keeps each
/// window's source integer keys. Associative arrays are rejected (only indexed arrays are
/// supported), non-array arguments are rejected, and so is a non-literal flag. The first argument
/// is re-inferred here to drive the return type; the registry already inferred every argument once
/// for side effects, and arity (2 or 3) is pre-validated.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    let Some(preserve) = literal_preserve_keys(cx.args.get(2)) else {
        // A flag known only at run time: an INDEXED source has two arms the backend can both
        // emit, so the call answers with their union and the branch picks one, the same shape
        // `array_slice()` and `array_reverse()` use. Twig's `CoreExtension::batch()` forwards an
        // untyped parameter here. A gradual source keeps the diagnostic: its two-argument form
        // answers through `__elephc_array_chunk_gradual`, a prelude function rather than a native
        // lowering, so both arms are not available at the branch.
        // A GRADUAL source answers through `__elephc_array_chunk_gradual_flagged`, whose chunks
        // carry the source keys and are therefore hashes. The type is the one that helper's body
        // infers, for the reason the two-argument arm below spells out.
        // A source that is not an indexed array at all answers through
        // `__elephc_array_chunk_gradual_flagged`, whose chunks carry the source keys and are
        // therefore hashes. `array<mixed>` deliberately stays on the native two-arm path here: the
        // redirect keeps it there too, and the two must agree about the result's representation.
        if array_arg_is_gradually_acceptable(&ty) && !matches!(ty, PhpType::Array(_)) {
            return Ok(PhpType::AssocArray {
                key: Box::new(PhpType::Mixed),
                value: Box::new(PhpType::Mixed),
            });
        }
        let PhpType::Array(elem_ty) = ty.clone() else {
            return Err(CompileError::new(
                cx.span,
                "array_chunk() preserve_keys argument must be a literal bool in AOT mode",
            ));
        };
        let preserved = PhpType::Array(Box::new(PhpType::AssocArray {
            key: Box::new(PhpType::Int),
            value: elem_ty.clone(),
        }));
        let renumbered = PhpType::Array(Box::new(PhpType::Array(elem_ty)));
        return Ok(cx.checker.normalize_union_type(vec![renumbered, preserved]));
    };
    match ty {
        PhpType::Array(elem_ty) if preserve => Ok(PhpType::Array(Box::new(PhpType::AssocArray {
            key: Box::new(PhpType::Int),
            value: elem_ty,
        }))),
        PhpType::Array(elem_ty) => Ok(PhpType::Array(Box::new(PhpType::Array(elem_ty)))),
        // Hash storage and a boxed Mixed cell answer through `__elephc_array_chunk_gradual`, which
        // the lowering substitutes for the two-argument form. The type here must be the one that
        // helper's body INFERS -- `array<array<mixed>>`, from its single `$result[] = $chunk`
        // site -- not merely a sound supertype. Answering `array<mixed>` instead left the call
        // site reading boxed cells where the callee had stored raw chunk pointers, and
        // `$pair[0]` came back as garbage while `count($pair)` stayed right.
        other if cx.args.len() == 2 && array_arg_is_gradually_acceptable(&other) => Ok(
            PhpType::Array(Box::new(PhpType::Array(Box::new(PhpType::Mixed)))),
        ),
        PhpType::AssocArray { .. } => Err(CompileError::new(
            cx.span,
            "array_chunk() argument must be indexed array",
        )),
        _ => Err(CompileError::new(
            cx.span,
            "array_chunk() first argument must be array",
        )),
    }
}
