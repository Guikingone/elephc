//! Purpose:
//! Dispatches AST statement nodes into EIR instructions and CFG terminators.
//!
//! Called from:
//! - `crate::ir_lower::function` for main, user functions, and methods.
//!
//! Key details:
//! - Every `StmtKind` variant has an explicit lowering branch.
//! - Structured control flow creates EIR blocks; complex PHP runtime behavior
//!   uses high-level opcodes with conservative effects.

use std::collections::HashSet;

use crate::ir::{
    BlockId, CmpPredicate, Immediate, IrType, LocalKind, LocalSlotId, Op, Ownership, SwitchCase,
    RuntimeCallTarget, Terminator,
};
use crate::ir_lower::context::{
    FinallyFrame, LoopCleanup, LoopFrame, LoweredValue, LoweringContext,
};
use crate::ir_lower::effects_lookup;
use crate::ir_lower::expr::{
    array_access_element_result_type, coerce_container_to_mixed_payload, coerce_to_int_at_span,
    index_expr_key_type, lower_array_access_from_lowered_receiver,
    lower_callable_array_for_assignment, lower_array_literal_with_expected_type,
    lower_closure_for_assignment, lower_expr,
    reflection_arg_array_binding_for_expr, reflection_class_binding_for_expr,
    reflection_function_binding_for_expr, reflection_method_binding_for_expr,
    reflection_property_binding_for_expr, static_callable_binding_for_expr,
    string_op_uses_scratch_storage, type_satisfies_array_access_for_ir,
};
use crate::names::{php_symbol_key, property_hook_set_method};
use crate::parser::ast::{
    is_compound_assignment_self_read, CatchClause, Expr, ExprKind, StaticReceiver, Stmt, StmtKind,
};
use crate::span::Span;
use crate::types::{PhpType, ThrowAccessKind};

mod statement_basics;
mod local_assignments;
mod conditionals;
mod loops;
mod array_write_core;
mod nested_array_writes;
mod array_write_storage;
mod typed_foreach;
mod switches;
mod includes;
mod exceptions;
mod control_exit;
mod declarations;
mod instance_property_writes;
mod static_property_writes;
mod property_array_writes;
mod metadata_control;
mod return_coercions;
mod static_property_helpers;

use statement_basics::*;
use local_assignments::*;
use conditionals::*;
use loops::*;
use array_write_core::*;
use nested_array_writes::*;
use array_write_storage::*;
use typed_foreach::*;
use switches::*;
use includes::*;
use exceptions::*;
use control_exit::*;
use declarations::*;
use instance_property_writes::*;
use static_property_writes::*;
use property_array_writes::*;
use metadata_control::*;
use return_coercions::*;
use static_property_helpers::*;

pub(crate) use control_exit::{lower_throw_access_error, lower_throw_access_error_expr};
pub(super) use array_write_core::{
    indexed_array_write_element_type, release_indexed_array_write_operand,
};
pub(super) use array_write_storage::{
    finish_indexed_array_local_write, prepare_indexed_array_local_write,
    ref_bound_mixed_indexed_array_write,
};

/// Lowers one AST statement into the current EIR insertion block.
pub(crate) fn lower_stmt(ctx: &mut LoweringContext<'_, '_>, stmt: &Stmt) {
    crate::strict_php::with_source_mode(stmt.source_mode, || {
        lower_stmt_in_current_source_mode(ctx, stmt);
    });
}

/// Lowers one statement after installing its physical source visibility profile.
fn lower_stmt_in_current_source_mode(ctx: &mut LoweringContext<'_, '_>, stmt: &Stmt) {
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
    lower_statement_concat_reset(ctx, stmt.span);
    match &stmt.kind {
        StmtKind::Echo(expr) => lower_echo(ctx, expr, stmt.span),
        StmtKind::Assign { name, value } => lower_assign(ctx, name, value, stmt.span),
        StmtKind::RefAssign { target, source } => lower_ref_assign(ctx, target, source, stmt.span),
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => lower_if(
            ctx,
            condition,
            then_body,
            elseif_clauses,
            else_body.as_deref(),
            stmt.span,
        ),
        StmtKind::IfDef {
            symbol,
            then_body,
            else_body,
        } => lower_ifdef(ctx, symbol, then_body, else_body.as_deref(), stmt.span),
        StmtKind::While { condition, body } => lower_while(ctx, condition, body, stmt.span),
        StmtKind::DoWhile { body, condition } => {
            lower_do_while(ctx, body, condition, stmt.span);
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => lower_for(
            ctx,
            init.as_deref(),
            condition.as_ref(),
            update.as_deref(),
            body,
            stmt.span,
        ),
        StmtKind::ArrayAssign {
            array,
            index,
            value,
        } => {
            lower_array_assign(ctx, array, index, value, stmt.span);
        }
        StmtKind::NestedArrayAssign { target, value } => {
            lower_nested_array_assign(ctx, target, value, stmt.span);
        }
        StmtKind::ArrayPush { array, value } => lower_array_push(ctx, array, value, stmt.span),
        StmtKind::TypedAssign {
            type_expr,
            name,
            value,
        } => lower_typed_assign(ctx, type_expr, name, value, stmt.span),
        StmtKind::Foreach {
            array,
            key_var,
            value_var,
            value_by_ref,
            body,
        } => lower_foreach(
            ctx,
            array,
            key_var.as_deref(),
            value_var,
            *value_by_ref,
            body,
            stmt.span,
        ),
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => lower_switch(ctx, subject, cases, default.as_deref()),
        StmtKind::Include {
            path,
            once,
            required,
        } => lower_include(ctx, path, *once, *required, stmt.span),
        StmtKind::IncludeOnceMark { label } => lower_include_once_mark(ctx, label, stmt.span),
        StmtKind::IncludeOnceGuard { label, body } => {
            lower_include_once_guard(ctx, label, body, stmt.span);
        }
        StmtKind::Throw(expr) => lower_throw(ctx, expr),
        StmtKind::Synthetic(body) => lower_block(ctx, body),
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => lower_try(ctx, try_body, catches, finally_body.as_deref(), stmt.span),
        StmtKind::Break(level) => lower_break(ctx, *level),
        StmtKind::Continue(level) => lower_continue(ctx, *level),
        StmtKind::ExprStmt(expr) => {
            let value = lower_expr(ctx, expr);
            release_expr_statement_result(ctx, value, expr.span);
        }
        StmtKind::NamespaceDecl { name: _ } => lower_noop(ctx, stmt.span),
        StmtKind::NamespaceBlock { name: _, body } => lower_block(ctx, body),
        StmtKind::UseDecl { imports: _ } => lower_noop(ctx, stmt.span),
        StmtKind::FunctionDecl { .. }
        | StmtKind::ClassDecl { .. }
        | StmtKind::EnumDecl { .. }
        | StmtKind::PackedClassDecl { .. }
        | StmtKind::InterfaceDecl { .. }
        | StmtKind::TraitDecl { .. }
        | StmtKind::ExternFunctionDecl { .. }
        | StmtKind::ExternClassDecl { .. }
        | StmtKind::ExternGlobalDecl { .. } => lower_noop(ctx, stmt.span),
        StmtKind::FunctionVariantGroup { name, variants } => {
            lower_function_variant_group(ctx, name, variants, stmt.span);
        }
        StmtKind::FunctionVariantMark { name, variant } => {
            lower_function_variant_mark(ctx, name, variant, stmt.span);
        }
        StmtKind::Return(value) => lower_return(ctx, value.as_ref(), stmt.span),
        StmtKind::ConstDecl { name, value } => lower_const_decl(ctx, name, value, stmt.span),
        StmtKind::ListUnpack { vars, value } => lower_list_unpack(ctx, vars, value, stmt.span),
        StmtKind::Global { vars } => lower_global(ctx, vars),
        StmtKind::StaticVar { name, init } => lower_static_var(ctx, name, init, stmt.span),
        StmtKind::PropertyAssign {
            object,
            property,
            value,
        } => lower_property_assign(ctx, object, property, value, stmt.span),
        StmtKind::StaticPropertyAssign {
            receiver,
            property,
            value,
        } => lower_static_property_assign(ctx, receiver, property, value, stmt.span),
        StmtKind::StaticPropertyArrayPush {
            receiver,
            property,
            value,
        } => lower_static_property_array_push(ctx, receiver, property, value, stmt.span),
        StmtKind::StaticPropertyArrayAssign {
            receiver,
            property,
            index,
            value,
        } => lower_static_property_array_assign(ctx, receiver, property, index, value, stmt.span),
        StmtKind::PropertyArrayPush {
            object,
            property,
            value,
        } => lower_property_array_push(ctx, object, property, value, stmt.span),
        StmtKind::PropertyArrayAssign {
            object,
            property,
            index,
            value,
        } => lower_property_array_assign(ctx, object, property, index, value, stmt.span),
    }
}
