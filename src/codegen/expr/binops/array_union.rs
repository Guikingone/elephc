//! Purpose:
//! Lowers PHP array union expressions and optimized empty-array cases.
//! Keeps operator-specific conversions and result register setup out of the dispatcher.
//!
//! Called from:
//! - `crate::codegen::expr::binops`
//!
//! Key details:
//! - Runtime calls and target instructions must preserve left/right evaluation order and scratch register assumptions.
//! - Mixed indexed/associative unions promote the indexed operand to a hash via `__rt_array_to_hash`,
//!   then reuse `__rt_hash_union` and `__rt_decref_hash` to release the promotion temporary.

use super::super::super::context::Context;
use super::super::super::data_section::DataSection;
use super::super::super::emit::Emitter;
use super::super::super::{abi, platform::Arch};
use super::super::{emit_expr, Expr, ExprKind, PhpType};

#[derive(Clone, Copy)]
enum UnionKind {
    IndexedIndexed,
    AssocAssoc,
    IndexedAssoc,
    AssocIndexed,
}

pub(super) fn is_array_union_candidate(left: &Expr, right: &Expr, ctx: &Context) -> bool {
    matches!(
        (
            super::super::super::functions::infer_contextual_type(left, ctx),
            super::super::super::functions::infer_contextual_type(right, ctx),
        ),
        (PhpType::Array(_), PhpType::Array(_))
            | (PhpType::AssocArray { .. }, PhpType::AssocArray { .. })
            | (PhpType::Array(_), PhpType::AssocArray { .. })
            | (PhpType::AssocArray { .. }, PhpType::Array(_))
    )
}

pub(super) fn emit_array_union_binop(
    left: &Expr,
    right: &Expr,
    emitter: &mut Emitter,
    ctx: &mut Context,
    data: &mut DataSection,
) -> PhpType {
    let left_static_ty = emit_expr(left, emitter, ctx, data);
    abi::emit_push_reg(emitter, abi::int_result_reg(emitter));                  // save the left array pointer while evaluating the right operand
    let right_static_ty = emit_expr(right, emitter, ctx, data);
    let result_ty = array_union_result_type(left, &left_static_ty, right, &right_static_ty);
    let union_kind = pick_union_kind(&left_static_ty, &right_static_ty);

    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("mov x1, x0");                                  // pass the right array pointer as the second runtime argument
            abi::emit_pop_reg(emitter, "x0");                                   // restore the left array pointer as the first runtime argument
        }
        Arch::X86_64 => {
            emitter.instruction("mov rsi, rax");                                // pass the right array pointer as the second runtime argument
            abi::emit_pop_reg(emitter, "rdi");                                  // restore the left array pointer as the first runtime argument
        }
    }

    match union_kind {
        UnionKind::IndexedIndexed => {
            abi::emit_call_label(emitter, "__rt_array_union");                  // compute PHP indexed-array union with numeric-key precedence
        }
        UnionKind::AssocAssoc => {
            abi::emit_call_label(emitter, "__rt_hash_union");                   // compute PHP associative-array union with left-key precedence
        }
        UnionKind::IndexedAssoc => {
            emit_indexed_plus_assoc_union(emitter);
        }
        UnionKind::AssocIndexed => {
            emit_assoc_plus_indexed_union(emitter);
        }
    }

    result_ty
}

fn pick_union_kind(left: &PhpType, right: &PhpType) -> UnionKind {
    match (left, right) {
        (PhpType::Array(_), PhpType::Array(_)) => UnionKind::IndexedIndexed,
        (PhpType::AssocArray { .. }, PhpType::AssocArray { .. }) => UnionKind::AssocAssoc,
        (PhpType::Array(_), PhpType::AssocArray { .. }) => UnionKind::IndexedAssoc,
        (PhpType::AssocArray { .. }, PhpType::Array(_)) => UnionKind::AssocIndexed,
        // Fallback when one side is statically erased to a coarser type (e.g. `Mixed`).
        // The hash representation is the more general one, so prefer that path.
        _ => UnionKind::AssocAssoc,
    }
}

fn emit_indexed_plus_assoc_union(emitter: &mut Emitter) {
    // Entry: x0/rdi = left (indexed), x1/rsi = right (assoc).
    // Promote the indexed left to a fresh hash, then run hash union, then decref the promotion temp.
    match emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg(emitter, "x1");                                  // spill the right associative array while promoting the indexed left
            abi::emit_call_label(emitter, "__rt_array_to_hash");                // promote the indexed left to a fresh hash with int keys 0..n-1
            abi::emit_pop_reg(emitter, "x1");                                   // restore the right associative array as the union right operand
            abi::emit_push_reg(emitter, "x0");                                  // save the promoted-left hash pointer for later decref cleanup
            abi::emit_call_label(emitter, "__rt_hash_union");                   // compute the union using the promoted-left hash and the right hash
            abi::emit_pop_reg(emitter, "x9");                                   // reload the promoted-left hash pointer from the spill slot
            abi::emit_push_reg(emitter, "x0");                                  // save the union result while we release the promotion temporary
            emitter.instruction("mov x0, x9");                                  // pass the promoted-left hash pointer to the decref helper
            abi::emit_call_label(emitter, "__rt_decref_hash");                  // release the throwaway promoted-left hash now that the union owns its entries
            abi::emit_pop_reg(emitter, "x0");                                   // restore the union result as the expression value
        }
        Arch::X86_64 => {
            abi::emit_push_reg(emitter, "rsi");                                 // spill the right associative array while promoting the indexed left
            abi::emit_call_label(emitter, "__rt_array_to_hash");                // promote the indexed left to a fresh hash with int keys 0..n-1
            abi::emit_pop_reg(emitter, "rsi");                                  // restore the right associative array as the union right operand
            abi::emit_push_reg(emitter, "rax");                                 // save the promoted-left hash pointer for later decref cleanup
            emitter.instruction("mov rdi, rax");                                // pass the promoted-left hash pointer as the union left operand
            abi::emit_call_label(emitter, "__rt_hash_union");                   // compute the union using the promoted-left hash and the right hash
            abi::emit_pop_reg(emitter, "r10");                                  // reload the promoted-left hash pointer into a scratch register
            abi::emit_push_reg(emitter, "rax");                                 // save the union result while we release the promotion temporary
            emitter.instruction("mov rax, r10");                                // x86_64 __rt_decref_hash reads its argument from rax, not rdi
            abi::emit_call_label(emitter, "__rt_decref_hash");                  // release the throwaway promoted-left hash now that the union owns its entries
            abi::emit_pop_reg(emitter, "rax");                                  // restore the union result as the expression value
        }
    }
}

fn emit_assoc_plus_indexed_union(emitter: &mut Emitter) {
    // Entry: x0/rdi = left (assoc), x1/rsi = right (indexed).
    // Promote the indexed right to a fresh hash, then run hash union, then decref the promotion temp.
    match emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg(emitter, "x0");                                  // spill the left associative array while promoting the indexed right
            emitter.instruction("mov x0, x1");                                  // move the indexed right into the promotion argument register
            abi::emit_call_label(emitter, "__rt_array_to_hash");                // promote the indexed right to a fresh hash with int keys 0..n-1
            emitter.instruction("mov x1, x0");                                  // place the promoted-right hash into the union second-argument register
            abi::emit_pop_reg(emitter, "x0");                                   // restore the left associative array as the union first operand
            abi::emit_push_reg(emitter, "x1");                                  // save the promoted-right hash pointer for later decref cleanup
            abi::emit_call_label(emitter, "__rt_hash_union");                   // compute the union using the left hash and the promoted-right hash
            abi::emit_pop_reg(emitter, "x9");                                   // reload the promoted-right hash pointer from the spill slot
            abi::emit_push_reg(emitter, "x0");                                  // save the union result while we release the promotion temporary
            emitter.instruction("mov x0, x9");                                  // pass the promoted-right hash pointer to the decref helper
            abi::emit_call_label(emitter, "__rt_decref_hash");                  // release the throwaway promoted-right hash now that the union owns its entries
            abi::emit_pop_reg(emitter, "x0");                                   // restore the union result as the expression value
        }
        Arch::X86_64 => {
            abi::emit_push_reg(emitter, "rdi");                                 // spill the left associative array while promoting the indexed right
            emitter.instruction("mov rdi, rsi");                                // move the indexed right into the promotion argument register
            abi::emit_call_label(emitter, "__rt_array_to_hash");                // promote the indexed right to a fresh hash with int keys 0..n-1
            emitter.instruction("mov rsi, rax");                                // place the promoted-right hash into the union second-argument register
            abi::emit_pop_reg(emitter, "rdi");                                  // restore the left associative array as the union first operand
            abi::emit_push_reg(emitter, "rsi");                                 // save the promoted-right hash pointer for later decref cleanup
            abi::emit_call_label(emitter, "__rt_hash_union");                   // compute the union using the left hash and the promoted-right hash
            abi::emit_pop_reg(emitter, "r10");                                  // reload the promoted-right hash pointer into a scratch register
            abi::emit_push_reg(emitter, "rax");                                 // save the union result while we release the promotion temporary
            emitter.instruction("mov rax, r10");                                // x86_64 __rt_decref_hash reads its argument from rax, not rdi
            abi::emit_call_label(emitter, "__rt_decref_hash");                  // release the throwaway promoted-right hash now that the union owns its entries
            abi::emit_pop_reg(emitter, "rax");                                  // restore the union result as the expression value
        }
    }
}

fn array_union_result_type(
    left_expr: &Expr,
    left: &PhpType,
    right_expr: &Expr,
    right: &PhpType,
) -> PhpType {
    match (left, right) {
        (PhpType::Array(_), PhpType::Array(_)) if is_empty_indexed_array_literal(left_expr) => {
            right.clone()
        }
        (PhpType::Array(_), PhpType::Array(_)) if is_empty_indexed_array_literal(right_expr) => {
            left.clone()
        }
        (PhpType::Array(left_elem), PhpType::Array(right_elem)) if left_elem == right_elem => {
            PhpType::Array(left_elem.clone())
        }
        (PhpType::Array(left_elem), PhpType::Array(_)) => PhpType::Array(left_elem.clone()),
        (
            PhpType::AssocArray {
                key: left_key,
                value: left_value,
            },
            PhpType::AssocArray {
                key: right_key,
                value: right_value,
            },
        ) => {
            let key = if left_key == right_key {
                left_key.clone()
            } else {
                Box::new(PhpType::Mixed)
            };
            let value = if left_value == right_value {
                left_value.clone()
            } else {
                Box::new(PhpType::Mixed)
            };
            PhpType::AssocArray { key, value }
        }
        (
            PhpType::Array(indexed_elem),
            PhpType::AssocArray {
                key: assoc_key,
                value: assoc_value,
            },
        ) => {
            if is_empty_indexed_array_literal(left_expr) {
                return PhpType::AssocArray {
                    key: assoc_key.clone(),
                    value: assoc_value.clone(),
                };
            }
            mixed_kind_union_assoc_type(indexed_elem, assoc_key, assoc_value)
        }
        (
            PhpType::AssocArray {
                key: assoc_key,
                value: assoc_value,
            },
            PhpType::Array(indexed_elem),
        ) => {
            if is_empty_indexed_array_literal(right_expr) {
                return PhpType::AssocArray {
                    key: assoc_key.clone(),
                    value: assoc_value.clone(),
                };
            }
            mixed_kind_union_assoc_type(indexed_elem, assoc_key, assoc_value)
        }
        _ => left.clone(),
    }
}

fn mixed_kind_union_assoc_type(
    indexed_elem: &PhpType,
    assoc_key: &PhpType,
    assoc_value: &PhpType,
) -> PhpType {
    let key = if *assoc_key == PhpType::Int {
        Box::new(PhpType::Int)
    } else {
        Box::new(PhpType::Mixed)
    };
    let value = if assoc_value == indexed_elem {
        Box::new(assoc_value.clone())
    } else {
        Box::new(PhpType::Mixed)
    };
    PhpType::AssocArray { key, value }
}

fn is_empty_indexed_array_literal(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::ArrayLiteral(elems) if elems.is_empty())
}
