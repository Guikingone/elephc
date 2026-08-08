//! Purpose:
//! Dispatches AST expression nodes into EIR values while preserving source-order
//! evaluation.
//!
//! Called from:
//! - `crate::ir_lower::stmt` and nested expression lowering.
//!
//! Key details:
//! - Simple scalar operations lower to concrete EIR arithmetic/string opcodes.
//! - Complex PHP runtime behavior lowers to high-level EIR opcodes with
//!   conservative effects until Phase 04 gives them target-specific meaning.

use crate::ir::{
    BlockId, CmpPredicate, Effects, Immediate, IrHeapKind, IrType, LocalKind, LocalSlotId,
    MixedNumericOp, Op, Ownership, Terminator, ValueId,
};
use crate::ir_lower::context::{
    value_ir_type, ClosureCapture, LoweredValue, LoweringContext, StaticCallableBinding,
};
use crate::ir_lower::effects_lookup;
use crate::ir_lower::function;
use crate::names::{php_symbol_key, property_hook_get_method, property_hook_set_method, Name};
use crate::parser::ast::{
    is_compound_assignment_self_read, BinOp, CallableTarget, CastType, Expr, ExprKind,
    InstanceOfTarget, MagicConstant, StaticReceiver, Stmt, StmtKind, TypeExpr, Visibility,
};
use crate::span::Span;
use crate::types::checker::builtins::canonical_builtin_function_name;
use crate::types::{
    checker::infer_expr_type_syntactic, merge_array_key_types, normalized_array_key_type,
    ExternFunctionSig, FunctionSig, PhpType, ReturnArgAlias, ThrowAccessKind,
};
use std::collections::HashSet;

mod constants;
mod nullsafe_chain;
mod scalar_literals;
mod numeric_binary;
mod string_concat;
mod comparisons;
mod unary_logic;
mod lazy_branches;
mod pipe;
mod assignments;
mod function_calls;
mod eval_barriers;
mod lazy_isset;
mod native_isset;
mod callable_probes;
mod descriptor_invoke;
mod descriptor_args;
mod static_array_callbacks;
mod callable_tracking;
mod callable_resolution;
mod unset;
mod array_builtin_args;
mod builtin_special_args;
mod call_arg_coercion;
mod positional_spreads;
mod named_args;
mod named_spreads;
mod variadic_args;
mod call_return_types;
mod indexed_array_literals;
mod assoc_array_literals;
mod match_expr;
mod array_access;
mod array_access_types;
mod ternary_cast;
mod closures;
mod closure_calls;
mod descriptor_calls;
mod object_construction;
mod property_access;
mod property_fetch_for_write;
mod method_calls;
mod reflection_class_calls;
mod reflection_method_calls;
mod reflection_property_calls;
mod reflection_filters;
mod reflection_constructors;
mod reflection_static_properties;
mod reflection_new_instance;
mod nullable_method_calls;
mod method_metadata;
mod static_method_calls;
mod scoped_values;
mod generators;
mod instanceof_coercions;
mod merge_temps;

use scalar_literals::*;
use numeric_binary::*;
use string_concat::*;
use comparisons::*;
use unary_logic::*;
use lazy_branches::*;
use pipe::*;
use assignments::*;
use function_calls::*;
use eval_barriers::*;
use lazy_isset::*;
use native_isset::*;
use callable_probes::*;
use descriptor_invoke::*;
use descriptor_args::*;
use static_array_callbacks::*;
use callable_tracking::*;
use callable_resolution::*;
use unset::*;
use array_builtin_args::*;
use builtin_special_args::*;
use call_arg_coercion::*;
use positional_spreads::*;
use named_args::*;
use named_spreads::*;
use variadic_args::*;
use indexed_array_literals::*;
use assoc_array_literals::*;
use match_expr::*;
use array_access::*;
use array_access_types::*;
use ternary_cast::*;
use closures::*;
use closure_calls::*;
use descriptor_calls::*;
use object_construction::*;
use property_access::*;
use method_calls::*;
use reflection_class_calls::*;
use reflection_method_calls::*;
use reflection_property_calls::*;
use reflection_filters::*;
use reflection_constructors::*;
use reflection_static_properties::*;
use reflection_new_instance::*;
use nullable_method_calls::*;
use method_metadata::*;
use static_method_calls::*;
use scoped_values::*;
use generators::*;
use instanceof_coercions::*;
use merge_temps::*;

pub(crate) use callable_resolution::{
    is_bound_closure_assignment_shape, lower_bound_closure_for_assignment,
};
pub(crate) use callable_tracking::{
    lower_callable_array_for_assignment, reflection_arg_array_binding_for_expr,
    reflection_class_binding_for_expr, reflection_function_binding_for_expr,
    reflection_method_binding_for_expr, reflection_property_binding_for_expr,
    static_callable_binding_for_expr,
};
#[allow(unused_imports)]
pub(crate) use callable_tracking::LoweredCallableArrayAssignment;
pub(crate) use closures::{body_contains_eval_call, lower_closure_for_assignment};
pub(crate) use indexed_array_literals::{
    array_literal_type_for_ir, lower_array_literal_with_expected_type,
};
pub(crate) use array_access::{
    array_access_element_result_type, index_expr_key_type,
    lower_array_access_from_lowered_receiver, lower_by_ref_foreach_element_source,
};
pub(crate) use array_access_types::type_satisfies_array_access_for_ir;
pub(crate) use instanceof_coercions::coerce_to_int_at_span;
pub(crate) use merge_temps::emit_bool_literal;
pub(crate) use property_access::{
    lower_ref_assign_array_elem, lower_ref_assign_call, lower_ref_assign_property,
};
pub(crate) use property_fetch_for_write::lower_by_ref_foreach_property_source;
pub(crate) use string_concat::string_op_uses_scratch_storage;
pub(super) use assoc_array_literals::{
    array_access_expr_value_type_for_ir, method_call_expr_type_for_ir,
    property_access_expr_type_for_ir,
};
pub(super) use call_return_types::call_return_type;
pub(super) use merge_temps::coerce_container_to_mixed_payload;
pub(super) use nullable_method_calls::lower_dynamic_method_call_with_receiver;
pub(super) use static_method_calls::static_method_call_expr_type_for_ir;

/// Lowers an expression and returns its EIR value.
pub(crate) fn lower_expr(ctx: &mut LoweringContext<'_, '_>, expr: &Expr) -> LoweredValue {
    if let Some(value) = nullsafe_chain::lower(ctx, expr) {
        return value;
    }

    match &expr.kind {
        // `IncludeValue` is a transient parser node fully expanded by the resolver;
        // it can never reach this pass.
        ExprKind::IncludeValue { .. } => unreachable!(
            "ExprKind::IncludeValue must be expanded by the resolver"
        ),
        ExprKind::StringLiteral(value) => lower_string_literal(ctx, value, expr),
        ExprKind::IntLiteral(value) => lower_int_literal(ctx, *value, expr),
        ExprKind::FloatLiteral(value) => lower_float_literal(ctx, *value, expr),
        ExprKind::BoolLiteral(value) => lower_bool_literal(ctx, *value, expr),
        ExprKind::Null => lower_null(ctx, expr),
        ExprKind::Variable(name) => ctx.load_local(name, Some(expr.span)),
        ExprKind::BinaryOp { left, op, right } => lower_binary(ctx, left, op, right, expr),
        ExprKind::InstanceOf { value, target } => lower_instanceof(ctx, value, target, expr),
        ExprKind::Negate(inner) => lower_numeric_unary(ctx, inner, Op::INeg, Op::FNeg, expr),
        ExprKind::Not(inner) => lower_not(ctx, inner, expr),
        ExprKind::BitNot(inner) => lower_int_unary(ctx, inner, Op::IBitNot, expr),
        ExprKind::Throw(inner) => lower_throw_expr(ctx, inner, expr),
        ExprKind::ErrorSuppress(inner) => lower_error_suppress(ctx, inner, expr),
        ExprKind::Print(inner) => lower_print(ctx, inner, expr),
        ExprKind::NullCoalesce { value, default } => {
            lower_null_coalesce(ctx, value, default, expr)
        }
        ExprKind::Pipe { value, callable } => lower_pipe(ctx, value, callable, expr),
        ExprKind::Assignment {
            target,
            value,
            result_target,
            prelude,
            conditional_value_temp,
        } => lower_assignment_expr(
            ctx,
            target,
            value,
            result_target.as_deref(),
            prelude,
            conditional_value_temp.as_deref(),
            expr,
        ),
        ExprKind::PreIncrement(name) => lower_inc_dec(ctx, name, true, false, expr),
        ExprKind::PostIncrement(name) => lower_inc_dec(ctx, name, true, true, expr),
        ExprKind::PreDecrement(name) => lower_inc_dec(ctx, name, false, false, expr),
        ExprKind::PostDecrement(name) => lower_inc_dec(ctx, name, false, true, expr),
        ExprKind::FunctionCall { name, args } => lower_function_call(ctx, name, args, expr),
        ExprKind::ArrayLiteral(items) => lower_array_literal(ctx, items, expr),
        ExprKind::ArrayLiteralAssoc(pairs) => lower_assoc_array_literal(ctx, pairs, expr),
        ExprKind::Match { subject, arms, default } => lower_match(ctx, subject, arms, default.as_deref(), expr),
        ExprKind::ArrayAccess { array, index } => lower_array_access(ctx, array, index, expr),
        ExprKind::Ternary { condition, then_expr, else_expr } => {
            lower_ternary(ctx, condition, then_expr, else_expr, expr)
        }
        ExprKind::ShortTernary { value, default } => {
            lower_short_ternary(ctx, value, default, expr)
        }
        ExprKind::Cast { target, expr: inner } => lower_cast(ctx, target, inner, expr),
        ExprKind::Closure {
            params,
            variadic,
            variadic_by_ref,
            return_type,
            body,
            captures,
            capture_refs,
            is_static,
            ..
        } => lower_closure(
            ctx,
            params,
            variadic.as_deref(),
            *variadic_by_ref,
            return_type.as_ref(),
            body,
            captures,
            capture_refs,
            expr,
            *is_static,
        ),
        ExprKind::NamedArg { value, .. } => lower_expr(ctx, value),
        ExprKind::Spread(inner) => lower_expr(ctx, inner),
        ExprKind::ClosureCall { var, args } => lower_closure_call(ctx, var, args, expr),
        ExprKind::ExprCall { callee, args } => lower_expr_call(ctx, callee, args, expr),
        ExprKind::ConstRef(name) => constants::lower_const_ref(ctx, name, expr),
        ExprKind::NewObject { class_name, args } => lower_new_object(ctx, class_name, args, expr),
        ExprKind::Clone(inner) => lower_clone(ctx, inner, expr),
        ExprKind::NewDynamic { name_expr, args } => {
            lower_new_dynamic(ctx, name_expr, args, expr)
        }
        ExprKind::NewDynamicObject { class_name, fallback_class, required_parent, args } => {
            lower_new_dynamic_object(ctx, class_name, fallback_class, required_parent, args, expr)
        }
        ExprKind::PropertyAccess { object, property } => lower_property_get(ctx, object, property, Op::PropGet, expr),
        ExprKind::DynamicPropertyAccess { object, property } => lower_dynamic_property_get(ctx, object, property, expr),
        ExprKind::NullsafePropertyAccess { object, property } => {
            lower_property_get(ctx, object, property, Op::NullsafePropGet, expr)
        }
        ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            lower_dynamic_property_get(ctx, object, property, expr)
        }
        ExprKind::StaticPropertyAccess { receiver, property } => {
            lower_static_property_get(ctx, receiver, property, expr)
        }
        ExprKind::MethodCall {
            object,
            method,
            args,
        } => lower_method_call(ctx, object, method, args, Op::MethodCall, expr),
        ExprKind::NullsafeMethodCall {
            object,
            method,
            args,
        } => lower_nullsafe_method_call(ctx, object, method, args, expr),
        ExprKind::NullsafeDynamicMethodCall { .. } => {
            unreachable!("nullsafe dynamic method calls are lowered as a nullsafe postfix chain")
        }
        ExprKind::StaticMethodCall {
            receiver,
            method,
            args,
        } => lower_static_method_call(ctx, receiver, method, args, expr),
        ExprKind::FirstClassCallable(target) => lower_first_class_callable(ctx, target, expr),
        ExprKind::This => ctx.load_local("this", Some(expr.span)),
        ExprKind::PtrCast { target_type, expr: inner } => lower_ptr_cast(ctx, target_type, inner, expr),
        ExprKind::BufferNew { element_type, len } => lower_buffer_new(ctx, element_type, len, expr),
        ExprKind::ClassConstant { receiver } => lower_class_constant(ctx, receiver, expr),
        ExprKind::ObjectClassName { object } => lower_object_class_name(ctx, object, expr),
        ExprKind::ScopedConstantAccess { receiver, name } => {
            lower_scoped_constant(ctx, receiver, name, expr)
        }
        ExprKind::NewScopedObject { receiver, args } => lower_new_scoped_object(ctx, receiver, args, expr),
        ExprKind::MagicConstant(kind) => lower_magic_constant(ctx, kind, expr),
        ExprKind::Yield { key, value } => lower_yield(ctx, key.as_deref(), value.as_deref(), expr),
        ExprKind::YieldFrom(inner) => lower_yield_from(ctx, inner, expr),
    }
}
