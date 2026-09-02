//! Purpose:
//! Computes fixed-point storage contracts for locals carried around loop back-edges.
//!
//! Called from:
//! - `crate::types::checker::stmt_check::control_flow` before checking a loop body.
//!
//! Key details:
//! - The analysis iterates assignment and array-growth evidence until local types stop changing.
//! - Array payload changes retain their container-specific conversions, while nullable locals
//!   that receive a non-null whole value across a back-edge use one boxed `Mixed` contract.
//! - EIR lowering consumes the checker-recorded contract instead of repeating expression
//!   inference, keeping non-literal RHSs and cascading promotions aligned across both layers.

use std::collections::HashSet;

use crate::parser::ast::{CastType, Expr, ExprKind, Stmt, StmtKind};
use crate::types::{
    merge_array_key_types, normalized_array_key_type, PhpType, TypeEnv,
};

/// Source recorded for a local assignment that may affect a later loop-carried array rebind.
enum AssignedValue<'a> {
    /// An ordinary assignment whose RHS can be inferred under the evolving environment.
    Expr(&'a Expr),
    /// A binding whose semantic type is declared by its enclosing control-flow construct.
    Known(PhpType),
    /// A binding without a statically available RHS, such as a `foreach` value.
    Opaque,
}

/// One local array write, including an optional explicit key expression.
struct ArrayWrite<'a> {
    /// Local receiving the element.
    name: &'a str,
    /// Explicit offset for `$array[$key] = value`, or `None` for append forms.
    index: Option<&'a Expr>,
    /// Value stored in the array.
    value: &'a Expr,
}

/// Computes stable storage types for locals already present at loop entry.
///
/// `infer_value` receives the evolving fixed-point environment, so a promotion can cascade
/// through intermediate locals and through later iterations. The result contains only locals
/// that require an up-front representation contract: indexed/associative arrays with boxed
/// `mixed` payloads, tagged nullable scalars, or whole-value `mixed` for a local crossing a
/// back-edge.
///
/// `ref_value_binding` names a `foreach ($source as &$value)` pair whose SOURCE is a plain local.
/// See [`apply_ref_value_binding_evidence`] for why what the body stores into `$value` is element
/// evidence against `$source`.
pub fn loop_carried_storage_types<'a>(
    body: &'a [Stmt],
    update: Option<&'a Stmt>,
    entry: &TypeEnv,
    ref_value_binding: Option<(&'a str, &'a str)>,
    infer_value: &mut dyn FnMut(&Expr, &TypeEnv) -> Option<PhpType>,
) -> Vec<(String, PhpType)> {
    let mut assignments = Vec::new();
    collect_value_assignments(body, &mut assignments);
    if let Some(update) = update {
        collect_value_assignment_stmt(update, &mut assignments);
    }

    let mut writes = Vec::new();
    collect_array_writes(body, &mut writes);
    if let Some(update) = update {
        collect_array_write_stmt(update, &mut writes);
    }

    let mut fixed = entry.clone();
    let mut whole_representation_sources = HashSet::new();
    loop {
        let previous = fixed.clone();
        apply_assignment_evidence(
            &assignments,
            &mut fixed,
            infer_value,
            &mut whole_representation_sources,
        );
        apply_ref_value_binding_evidence(
            ref_value_binding,
            &assignments,
            &mut fixed,
            infer_value,
        );
        apply_array_write_evidence(&writes, &mut fixed, infer_value);
        if fixed == previous {
            break;
        }
    }

    let mut contracts = entry
        .iter()
        .filter_map(|(name, entry_ty)| {
            let fixed_ty = fixed.get(name)?;
            representation_contract(
                entry_ty,
                fixed_ty,
                whole_representation_sources.contains(name),
            )
                .map(|contract| (name.clone(), contract))
        })
        .collect::<Vec<_>>();
    contracts.sort_by(|left, right| left.0.cmp(&right.0));
    contracts
}

/// Carries a by-REFERENCE `foreach` value variable's fixed-point type into the iterated array's
/// element type.
///
/// `foreach ($chunks as &$chunk) { $chunk = new Chunk($chunk); }` writes THROUGH the reference into
/// `$chunks`, so the loop's element type has to accommodate whatever the body stores — exactly as
/// `$chunks[$k] = new Chunk(…)` would. Nothing else in this analysis knows that: the assignment is
/// syntactically a plain local store, and `collect_array_writes` only sees writes that NAME a
/// container. Without this the fixed point left `$chunks` at its entry `array<string>` while the
/// body stored an object into it, and the checker reported `cannot reassign $chunk from string to
/// Chunk` on code `php -n` runs (`Symfony\Component\String\ByteString::split` and its
/// `CodePointString` twin are the shape).
///
/// The stored types are re-derived here rather than read off the value variable's joined entry in
/// the fixed-point environment, because that entry cannot be trusted for this question:
/// `apply_assignment_evidence` ends its inference with `.unwrap_or(PhpType::Mixed)`, so a value it
/// has no answer for is indistinguishable from one that really is `mixed`, and boxing an array on
/// the strength of a fallback is exactly the wrong direction. An expression this analysis cannot
/// type is therefore NOT evidence at all — the same "only what can be typed exactly" discipline
/// `mixed_storage_scan` follows, and the safe one: missing a real widening costs the pre-existing
/// hard error, never a wrong representation.
///
/// It stays CONSERVATIVE where the checker is. `infer_type` answers `Mixed` for `int * int` —
/// PHP's arithmetic overflows to float — so `foreach ($nums as &$n) { $n = $n * 10; }` over an
/// `array<int>` really does box its payload. That is not a new answer: `$nums[$i] = $nums[$i] * 10`
/// in a loop has always boxed through `apply_array_write_evidence`, from the identical inference.
/// The two rules give the same answer to the same question by construction, which is the point of
/// routing this through the same join.
///
/// Applied per ITERATION, right after the assignment evidence, so a payload promotion can still
/// cascade — the array's new element type is what the next iteration's reads of `$chunk` see.
///
/// The join is the payload join, never the KEY one: a by-reference `foreach` visits the keys the
/// array already has, so it can neither turn a packed array into a hash nor renumber a hash.
/// A source that is not a plain local (`$this->items`) is not passed in at all — the caller can
/// neither read nor re-type such a place through this contract, and a target with no expression
/// behind it (a `RefAssign`, a `list()` element — [`AssignedValue::Opaque`]) has nothing to infer.
fn apply_ref_value_binding_evidence<'a>(
    ref_value_binding: Option<(&'a str, &'a str)>,
    assignments: &[(&'a str, AssignedValue<'a>)],
    env: &mut TypeEnv,
    infer_value: &mut dyn FnMut(&Expr, &TypeEnv) -> Option<PhpType>,
) {
    let Some((source_local, value_var)) = ref_value_binding else {
        return;
    };
    let mut stored: Option<PhpType> = None;
    for (name, source) in assignments {
        if *name != value_var {
            continue;
        }
        let AssignedValue::Expr(value) = source else {
            continue;
        };
        let Some(value_ty) = infer_storage_value_type(value, env, infer_value)
            .or_else(|| precise_scalar_expr_type(value))
        else {
            continue;
        };
        stored = Some(match stored {
            None => value_ty,
            Some(previous) => join_array_payload_type(previous, value_ty),
        });
    }
    let Some(stored) = stored else {
        return;
    };
    let updated = match env.get(source_local).cloned() {
        Some(PhpType::Array(element)) => {
            PhpType::Array(Box::new(join_array_payload_type(*element, stored)))
        }
        Some(PhpType::AssocArray { key, value }) => PhpType::AssocArray {
            key,
            value: Box::new(join_array_payload_type(*value, stored)),
        },
        _ => return,
    };
    env.insert(source_local.to_string(), updated);
}

/// Applies every collected local assignment monotonically to one fixed-point iteration.
fn apply_assignment_evidence(
    assignments: &[(&str, AssignedValue<'_>)],
    env: &mut TypeEnv,
    infer_value: &mut dyn FnMut(&Expr, &TypeEnv) -> Option<PhpType>,
    whole_representation_sources: &mut HashSet<String>,
) {
    for (name, source) in assignments {
        let incoming = match source {
            AssignedValue::Expr(expr) => infer_storage_value_type(expr, env, infer_value)
                .or_else(|| precise_scalar_expr_type(expr))
                .unwrap_or(PhpType::Mixed),
            AssignedValue::Known(ty) => ty.clone(),
            AssignedValue::Opaque => PhpType::Mixed,
        };
        let proves_whole_representation_change =
            env.get(*name).is_some_and(|existing| match source {
                AssignedValue::Opaque => false,
                AssignedValue::Expr(_) | AssignedValue::Known(_) if is_array_like(existing) => {
                    matches!(incoming.codegen_repr(), PhpType::Mixed | PhpType::Union(_))
                }
                AssignedValue::Expr(_) | AssignedValue::Known(_) => {
                    matches!(existing, PhpType::Void)
                        && !matches!(&incoming, PhpType::Void | PhpType::Never)
                }
            });
        if proves_whole_representation_change {
            whole_representation_sources.insert((*name).to_string());
        }
        let merged = env
            .get(*name)
            .cloned()
            .map(|existing| join_loop_flow_type(existing, incoming.clone()))
            .unwrap_or(incoming);
        env.insert((*name).to_string(), merged);
    }
}

/// Applies indexed-array growth and element-write evidence to one fixed-point iteration.
fn apply_array_write_evidence(
    writes: &[ArrayWrite<'_>],
    env: &mut TypeEnv,
    infer_value: &mut dyn FnMut(&Expr, &TypeEnv) -> Option<PhpType>,
) {
    for write in writes {
        let incoming = infer_storage_value_type(write.value, env, infer_value)
            .or_else(|| precise_scalar_expr_type(write.value))
            .unwrap_or(PhpType::Mixed);
        let Some(current) = env.get(write.name).cloned() else {
            continue;
        };
        let key_type = write.index.map(|index| {
            let inferred = infer_storage_value_type(index, env, infer_value)
                .or_else(|| precise_scalar_expr_type(index))
                .unwrap_or(PhpType::Mixed);
            normalized_array_key_type(index, inferred)
        });
        let forces_hash = write.index.is_some_and(|index| {
            matches!(key_type.as_ref(), Some(PhpType::Str))
                && matches!(index.kind, ExprKind::StringLiteral(_) | ExprKind::Null)
        });
        let updated = match current {
            PhpType::Array(element) if forces_hash => {
                let was_empty = matches!(element.as_ref(), PhpType::Never);
                PhpType::AssocArray {
                    key: Box::new(if was_empty {
                        key_type.unwrap_or(PhpType::Mixed)
                    } else {
                        merge_array_key_types(
                            PhpType::Int,
                            key_type.unwrap_or(PhpType::Mixed),
                        )
                    }),
                    value: Box::new(if was_empty {
                        incoming
                    } else {
                        join_array_payload_type(*element, incoming)
                    }),
                }
            }
            PhpType::Array(element) => {
                PhpType::Array(Box::new(join_array_payload_type(*element, incoming)))
            }
            PhpType::AssocArray { key, value } => PhpType::AssocArray {
                key: Box::new(merge_array_key_types(
                    *key,
                    key_type.unwrap_or(PhpType::Int),
                )),
                value: Box::new(join_array_payload_type(*value, incoming)),
            },
            other => other,
        };
        env.insert(write.name.to_string(), updated);
    }
}

/// Infers the representation an expression materializes under the evolving loop environment.
///
/// Semantic checker types are authoritative for ordinary expressions, but array literals need
/// storage-aware element inference: direct variables and array reads can change representation
/// as the fixed point evolves, while the remaining literal elements follow the same syntactic
/// fallback used by EIR literal lowering.
fn infer_storage_value_type(
    expr: &Expr,
    env: &TypeEnv,
    infer_value: &mut dyn FnMut(&Expr, &TypeEnv) -> Option<PhpType>,
) -> Option<PhpType> {
    match &expr.kind {
        ExprKind::Variable(name) => env.get(name).cloned().or_else(|| infer_value(expr, env)),
        ExprKind::ArrayLiteral(items) => {
            let mut element = PhpType::Never;
            for item in items {
                let next = infer_array_literal_element_storage(item, env, infer_value);
                element = merge_materialized_array_element(element, next);
            }
            Some(PhpType::Array(Box::new(element)))
        }
        ExprKind::ArrayLiteralAssoc(entries) => {
            let mut key = PhpType::Never;
            let mut value = PhpType::Never;
            for (key_expr, value_expr) in entries {
                let next_key = infer_value(key_expr, env)
                    .unwrap_or_else(|| {
                        crate::types::checker::infer_expr_type_syntactic(key_expr)
                    })
                    .codegen_repr();
                let next_value =
                    infer_array_literal_element_storage(value_expr, env, infer_value);
                key = merge_materialized_array_element(key, next_key);
                value = merge_materialized_array_element(value, next_value);
            }
            Some(PhpType::AssocArray {
                key: Box::new(key),
                value: Box::new(value),
            })
        }
        ExprKind::ErrorSuppress(inner) => infer_storage_value_type(inner, env, infer_value),
        ExprKind::Ternary {
            then_expr,
            else_expr,
            ..
        } => {
            let then_ty = infer_storage_value_type(then_expr, env, infer_value)?;
            let else_ty = infer_storage_value_type(else_expr, env, infer_value)?;
            Some(join_loop_flow_type(then_ty, else_ty))
        }
        _ => infer_value(expr, env),
    }
}

/// Infers one indexed/associative literal element's concrete runtime storage representation.
fn infer_array_literal_element_storage(
    item: &Expr,
    env: &TypeEnv,
    infer_value: &mut dyn FnMut(&Expr, &TypeEnv) -> Option<PhpType>,
) -> PhpType {
    match &item.kind {
        ExprKind::Variable(name) => env
            .get(name)
            .cloned()
            .or_else(|| infer_value(item, env))
            .unwrap_or(PhpType::Mixed)
            .codegen_repr(),
        ExprKind::ArrayAccess { array, .. } => {
            let source = infer_storage_value_type(array, env, infer_value)
                .or_else(|| infer_value(array, env))
                .unwrap_or(PhpType::Mixed)
                .codegen_repr();
            match source {
                PhpType::Array(element) => array_read_storage_type(*element),
                PhpType::AssocArray { value, .. } => array_read_storage_type(*value),
                PhpType::Str => PhpType::Str,
                _ => PhpType::Mixed,
            }
        }
        ExprKind::ArrayLiteral(_) | ExprKind::ArrayLiteralAssoc(_) => {
            infer_storage_value_type(item, env, infer_value)
                .unwrap_or(PhpType::Mixed)
                .codegen_repr()
        }
        ExprKind::Spread(inner) => match infer_storage_value_type(inner, env, infer_value)
            .unwrap_or(PhpType::Mixed)
            .codegen_repr()
        {
            PhpType::Array(element) => *element,
            PhpType::AssocArray { value, .. } => *value,
            _ => PhpType::Mixed,
        },
        ExprKind::FunctionCall { .. }
        | ExprKind::MethodCall { .. }
        | ExprKind::StaticMethodCall { .. }
        | ExprKind::PropertyAccess { .. }
        | ExprKind::ConstRef(_) => infer_value(item, env)
            .unwrap_or_else(|| crate::types::checker::infer_expr_type_syntactic(item))
            .codegen_repr(),
        _ => crate::types::checker::infer_expr_type_syntactic(item).codegen_repr(),
    }
}

/// Adds the miss-capable tagged representation used by integer array reads on tagged targets.
fn array_read_storage_type(element: PhpType) -> PhpType {
    if crate::codegen::sentinels::null_repr_is_tagged()
        && matches!(element.codegen_repr(), PhpType::Int)
    {
        PhpType::TaggedScalar
    } else {
        element.codegen_repr()
    }
}

/// Merges two materialized literal element representations.
fn merge_materialized_array_element(left: PhpType, right: PhpType) -> PhpType {
    if matches!(left, PhpType::Never) {
        return right;
    }
    if matches!(right, PhpType::Never) {
        return left;
    }
    if left.codegen_repr() == right.codegen_repr() {
        left
    } else {
        PhpType::Mixed
    }
}

/// Joins two possible values of a local on the loop-storage lattice.
fn join_loop_flow_type(existing: PhpType, incoming: PhpType) -> PhpType {
    if existing == incoming {
        return existing;
    }
    match (existing, incoming) {
        (PhpType::Array(left), PhpType::Array(right)) => {
            PhpType::Array(Box::new(join_array_payload_type(*left, *right)))
        }
        (
            PhpType::AssocArray {
                key: left_key,
                value: left_value,
            },
            PhpType::AssocArray {
                key: right_key,
                value: right_value,
            },
        ) if left_key.codegen_repr() == right_key.codegen_repr() => PhpType::AssocArray {
            key: left_key,
            value: Box::new(join_array_payload_type(*left_value, *right_value)),
        },
        (PhpType::Array(element), PhpType::AssocArray { key, value })
        | (PhpType::AssocArray { key, value }, PhpType::Array(element)) => {
            let array_was_empty = matches!(element.as_ref(), PhpType::Never);
            PhpType::AssocArray {
                key: if array_was_empty {
                    key
                } else {
                    Box::new(merge_array_key_types(PhpType::Int, *key))
                },
                value: Box::new(if array_was_empty {
                    *value
                } else {
                    join_array_payload_type(*element, *value)
                }),
            }
        }
        (PhpType::Array(array), PhpType::Void | PhpType::Never)
        | (PhpType::Void | PhpType::Never, PhpType::Array(array)) => PhpType::Array(array),
        (PhpType::AssocArray { key, value }, PhpType::Void | PhpType::Never)
        | (PhpType::Void | PhpType::Never, PhpType::AssocArray { key, value }) => {
            PhpType::AssocArray { key, value }
        }
        (left, right) if left.codegen_repr() == right.codegen_repr() => left,
        (left, right) => merge_loop_union_members(left, right),
    }
}

/// Builds a stable semantic union for loop-carried whole-value representation changes.
///
/// Retaining members such as `null|ArrayAccessObject` lets flow narrowing recover the concrete
/// arm inside a truthy branch while `codegen_repr()` still selects one boxed `Mixed` frame slot.
/// Existing unions are flattened and duplicate members are removed so the fixed-point converges.
fn merge_loop_union_members(left: PhpType, right: PhpType) -> PhpType {
    let mut members = Vec::new();
    for ty in [left, right] {
        match ty {
            PhpType::Union(nested) => {
                for member in nested {
                    if !members.contains(&member) {
                        members.push(member);
                    }
                }
            }
            member if !matches!(member, PhpType::Never) && !members.contains(&member) => {
                members.push(member);
            }
            _ => {}
        }
    }
    match members.len() {
        0 => PhpType::Never,
        1 => members.pop().unwrap(),
        _ => PhpType::Union(members),
    }
}

/// Joins array payload types so every live element has one stable runtime representation.
fn join_array_payload_type(existing: PhpType, incoming: PhpType) -> PhpType {
    if existing == incoming || existing.codegen_repr() == incoming.codegen_repr() {
        return existing;
    }
    if matches!(existing, PhpType::Never) {
        return incoming;
    }
    if matches!(incoming, PhpType::Never) {
        return existing;
    }
    PhpType::Mixed
}

/// Returns the concrete pre-loop storage contract required by a fixed-point result.
fn representation_contract(
    entry: &PhpType,
    fixed: &PhpType,
    has_whole_representation_source: bool,
) -> Option<PhpType> {
    match (entry.codegen_repr(), fixed.codegen_repr()) {
        (PhpType::Array(entry_element), PhpType::Array(fixed_element))
            if entry_element.codegen_repr() != PhpType::Mixed
                && fixed_element.codegen_repr() == PhpType::Mixed =>
        {
            Some(PhpType::Array(Box::new(PhpType::Mixed)))
        }
        (
            PhpType::AssocArray {
                value: entry_value,
                ..
            },
            PhpType::AssocArray {
                key: fixed_key,
                value: fixed_value,
            },
        ) if entry_value.codegen_repr() != PhpType::Mixed
            && fixed_value.codegen_repr() == PhpType::Mixed =>
        {
            Some(PhpType::AssocArray {
                key: fixed_key,
                value: Box::new(PhpType::Mixed),
            })
        }
        (PhpType::Array(_), fixed @ PhpType::AssocArray { .. }) => Some(fixed),
        (PhpType::Void | PhpType::Never, PhpType::TaggedScalar)
            if has_whole_representation_source =>
        {
            Some(fixed.clone())
        }
        (entry_repr, PhpType::Mixed)
            if entry_repr != PhpType::Mixed && has_whole_representation_source =>
        {
            Some(if is_array_like(entry) {
                PhpType::Mixed
            } else {
                fixed.clone()
            })
        }
        _ => None,
    }
}

/// Returns whether a loop-entry type is an indexed or associative array local.
fn is_array_like(ty: &PhpType) -> bool {
    matches!(
        ty.codegen_repr(),
        PhpType::Array(_) | PhpType::AssocArray { .. }
    )
}

/// Returns a self-evident scalar type when semantic inference is unavailable.
fn precise_scalar_expr_type(value: &Expr) -> Option<PhpType> {
    match &value.kind {
        ExprKind::IntLiteral(_) => Some(PhpType::Int),
        ExprKind::FloatLiteral(_) => Some(PhpType::Float),
        ExprKind::StringLiteral(_) => Some(PhpType::Str),
        ExprKind::BoolLiteral(_) => Some(PhpType::Bool),
        ExprKind::Null => Some(PhpType::Void),
        ExprKind::Cast { target, .. } => match target {
            CastType::Int => Some(PhpType::Int),
            CastType::Float => Some(PhpType::Float),
            CastType::String => Some(PhpType::Str),
            CastType::Bool => Some(PhpType::Bool),
            CastType::Array | CastType::Object => None,
        },
        ExprKind::ErrorSuppress(inner) => precise_scalar_expr_type(inner),
        _ => None,
    }
}

/// Collects local assignments from every statement in source traversal order.
fn collect_value_assignments<'a>(
    statements: &'a [Stmt],
    out: &mut Vec<(&'a str, AssignedValue<'a>)>,
) {
    for statement in statements {
        collect_value_assignment_stmt(statement, out);
    }
}

/// Collects local assignments from one statement and its executable nested bodies.
fn collect_value_assignment_stmt<'a>(
    statement: &'a Stmt,
    out: &mut Vec<(&'a str, AssignedValue<'a>)>,
) {
    match &statement.kind {
        StmtKind::Assign { name, value } | StmtKind::TypedAssign { name, value, .. } => {
            collect_value_assignments_from_expr(value, out);
            out.push((name, AssignedValue::Expr(value)));
        }
        StmtKind::RefAssign { target, source } => {
            collect_value_assignments_from_expr(source, out);
            out.push((target, AssignedValue::Opaque));
        }
        StmtKind::ListUnpack { vars, value } => {
            collect_value_assignments_from_expr(value, out);
            out.extend(
                vars.iter()
                    .map(|name| (name.as_str(), AssignedValue::Opaque)),
            );
        }
        StmtKind::ArrayAssign { index, value, .. } => {
            collect_value_assignments_from_expr(index, out);
            collect_value_assignments_from_expr(value, out);
        }
        StmtKind::NestedArrayAssign { target, value } => {
            collect_value_assignments_from_expr(target, out);
            collect_value_assignments_from_expr(value, out);
        }
        StmtKind::ArrayPush { value, .. } => {
            collect_value_assignments_from_expr(value, out);
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyRefAssign { object, source: value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            collect_value_assignments_from_expr(object, out);
            collect_value_assignments_from_expr(value, out);
        }
        StmtKind::PropertyArrayAssign {
            object,
            index,
            value,
            ..
        } => {
            collect_value_assignments_from_expr(object, out);
            collect_value_assignments_from_expr(index, out);
            collect_value_assignments_from_expr(value, out);
        }
        StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => {
            collect_value_assignments_from_expr(value, out);
        }
        StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            collect_value_assignments_from_expr(index, out);
            collect_value_assignments_from_expr(value, out);
        }
        StmtKind::StaticPropertyElementRefAssign { index, source, .. } => {
            collect_value_assignments_from_expr(index, out);
            collect_value_assignments_from_expr(source, out);
        }
        StmtKind::Foreach {
            array,
            key_var,
            value_var,
            body,
            ..
        } => {
            collect_value_assignments_from_expr(array, out);
            if let Some(key_var) = key_var {
                out.push((key_var, AssignedValue::Opaque));
            }
            out.push((value_var, AssignedValue::Opaque));
            collect_value_assignments(body, out);
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            collect_value_assignments_from_expr(condition, out);
            collect_value_assignments(then_body, out);
            for (condition, body) in elseif_clauses {
                collect_value_assignments_from_expr(condition, out);
                collect_value_assignments(body, out);
            }
            if let Some(else_body) = else_body {
                collect_value_assignments(else_body, out);
            }
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            collect_value_assignments(then_body, out);
            if let Some(else_body) = else_body {
                collect_value_assignments(else_body, out);
            }
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { condition, body } => {
            collect_value_assignments_from_expr(condition, out);
            collect_value_assignments(body, out);
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            if let Some(init) = init {
                collect_value_assignment_stmt(init, out);
            }
            if let Some(condition) = condition {
                collect_value_assignments_from_expr(condition, out);
            }
            collect_value_assignments(body, out);
            if let Some(update) = update {
                collect_value_assignment_stmt(update, out);
            }
        }
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            collect_value_assignments_from_expr(subject, out);
            for (values, body) in cases {
                for value in values {
                    collect_value_assignments_from_expr(value, out);
                }
                collect_value_assignments(body, out);
            }
            if let Some(default) = default {
                collect_value_assignments(default, out);
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            collect_value_assignments(try_body, out);
            for catch in catches {
                if let Some(variable) = &catch.variable {
                    let catch_type = if catch.exception_types.len() == 1 {
                        PhpType::Object(
                            catch.exception_types[0]
                                .as_str()
                                .trim_start_matches('\\')
                                .to_string(),
                        )
                    } else {
                        PhpType::Object("Throwable".to_string())
                    };
                    out.push((variable, AssignedValue::Known(catch_type)));
                }
                collect_value_assignments(&catch.body, out);
            }
            if let Some(finally_body) = finally_body {
                collect_value_assignments(finally_body, out);
            }
        }
        StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::Synthetic(body) => collect_value_assignments(body, out),
        StmtKind::ExprStmt(expr)
        | StmtKind::Echo(expr)
        | StmtKind::Throw(expr)
        | StmtKind::Include { path: expr, .. }
        | StmtKind::ConstDecl { value: expr, .. }
        | StmtKind::StaticVar { init: expr, .. } => {
            collect_value_assignments_from_expr(expr, out);
        }
        StmtKind::Return(Some(expr)) => collect_value_assignments_from_expr(expr, out),
        _ => {}
    }
}

/// Collects assignment expressions and increment/decrement writes from an expression tree.
fn collect_value_assignments_from_expr<'a>(
    expr: &'a Expr,
    out: &mut Vec<(&'a str, AssignedValue<'a>)>,
) {
    match &expr.kind {
        ExprKind::Assignment {
            target,
            value,
            prelude,
            result_target,
            ..
        } => {
            collect_value_assignments(prelude, out);
            collect_value_assignments_from_expr(value, out);
            if let ExprKind::Variable(name) = &target.kind {
                out.push((name, AssignedValue::Expr(value)));
            }
            collect_value_assignments_from_expr(target, out);
            if let Some(result_target) = result_target {
                collect_value_assignments_from_expr(result_target, out);
            }
        }
        ExprKind::PreIncrement(name)
        | ExprKind::PostIncrement(name)
        | ExprKind::PreDecrement(name)
        | ExprKind::PostDecrement(name) => out.push((name, AssignedValue::Opaque)),
        _ => visit_child_expressions(expr, &mut |child| {
            collect_value_assignments_from_expr(child, out)
        }),
    }
}

/// Collects local indexed/associative array element writes from executable statements.
fn collect_array_writes<'a>(statements: &'a [Stmt], out: &mut Vec<ArrayWrite<'a>>) {
    for statement in statements {
        collect_array_write_stmt(statement, out);
    }
}

/// Collects array growth/write sites from one statement and its nested bodies.
fn collect_array_write_stmt<'a>(statement: &'a Stmt, out: &mut Vec<ArrayWrite<'a>>) {
    match &statement.kind {
        StmtKind::ArrayPush { array, value } => {
            out.push(ArrayWrite {
                name: array,
                index: None,
                value,
            });
            collect_growth_calls_from_expr(value, out);
        }
        StmtKind::ArrayAssign {
            array,
            index,
            value,
        } => {
            out.push(ArrayWrite {
                name: array,
                index: Some(index),
                value,
            });
            collect_growth_calls_from_expr(index, out);
            collect_growth_calls_from_expr(value, out);
        }
        StmtKind::NestedArrayAssign { target, value } => {
            collect_growth_calls_from_expr(target, out);
            collect_growth_calls_from_expr(value, out);
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyRefAssign { object, source: value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            collect_growth_calls_from_expr(object, out);
            collect_growth_calls_from_expr(value, out);
        }
        StmtKind::PropertyArrayAssign {
            object,
            index,
            value,
            ..
        } => {
            collect_growth_calls_from_expr(object, out);
            collect_growth_calls_from_expr(index, out);
            collect_growth_calls_from_expr(value, out);
        }
        StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => {
            collect_growth_calls_from_expr(value, out);
        }
        StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            collect_growth_calls_from_expr(index, out);
            collect_growth_calls_from_expr(value, out);
        }
        StmtKind::StaticPropertyElementRefAssign { index, source, .. } => {
            collect_growth_calls_from_expr(index, out);
            collect_growth_calls_from_expr(source, out);
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            collect_growth_calls_from_expr(condition, out);
            collect_array_writes(then_body, out);
            for (condition, body) in elseif_clauses {
                collect_growth_calls_from_expr(condition, out);
                collect_array_writes(body, out);
            }
            if let Some(else_body) = else_body {
                collect_array_writes(else_body, out);
            }
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            collect_array_writes(then_body, out);
            if let Some(else_body) = else_body {
                collect_array_writes(else_body, out);
            }
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { condition, body } => {
            collect_growth_calls_from_expr(condition, out);
            collect_array_writes(body, out);
        }
        StmtKind::Foreach { array, body, .. } => {
            collect_growth_calls_from_expr(array, out);
            collect_array_writes(body, out);
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            if let Some(init) = init {
                collect_array_write_stmt(init, out);
            }
            if let Some(condition) = condition {
                collect_growth_calls_from_expr(condition, out);
            }
            collect_array_writes(body, out);
            if let Some(update) = update {
                collect_array_write_stmt(update, out);
            }
        }
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            collect_growth_calls_from_expr(subject, out);
            for (values, body) in cases {
                for value in values {
                    collect_growth_calls_from_expr(value, out);
                }
                collect_array_writes(body, out);
            }
            if let Some(default) = default {
                collect_array_writes(default, out);
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            collect_array_writes(try_body, out);
            for catch in catches {
                collect_array_writes(&catch.body, out);
            }
            if let Some(finally_body) = finally_body {
                collect_array_writes(finally_body, out);
            }
        }
        StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::Synthetic(body) => collect_array_writes(body, out),
        StmtKind::Assign { value, .. }
        | StmtKind::TypedAssign { value, .. }
        | StmtKind::ConstDecl { value, .. }
        | StmtKind::ListUnpack { value, .. }
        | StmtKind::StaticVar { init: value, .. }
        | StmtKind::Echo(value)
        | StmtKind::ExprStmt(value)
        | StmtKind::Throw(value)
        | StmtKind::Include { path: value, .. } => collect_growth_calls_from_expr(value, out),
        StmtKind::Return(Some(value)) => collect_growth_calls_from_expr(value, out),
        _ => {}
    }
}

/// Collects `array_push($local, ...)` sites from an expression tree.
fn collect_growth_calls_from_expr<'a>(expr: &'a Expr, out: &mut Vec<ArrayWrite<'a>>) {
    if let ExprKind::FunctionCall { name, args } = &expr.kind {
        if name.as_str().eq_ignore_ascii_case("array_push") {
            if let Some(array_name) = array_push_target_name(args) {
                out.extend(args.iter().skip(1).map(|argument| ArrayWrite {
                    name: array_name,
                    index: None,
                    value: call_arg_value(argument),
                }));
            }
        }
    }
    visit_child_expressions(expr, &mut |child| {
        collect_growth_calls_from_expr(child, out)
    });
}

/// Visits direct executable child expressions without descending into closure bodies.
fn visit_child_expressions<'a>(expr: &'a Expr, visitor: &mut dyn FnMut(&'a Expr)) {
    match &expr.kind {
        ExprKind::BinaryOp { left, right, .. }
        | ExprKind::NullCoalesce {
            value: left,
            default: right,
        }
        | ExprKind::ShortTernary {
            value: left,
            default: right,
        }
        | ExprKind::Pipe {
            value: left,
            callable: right,
        } => {
            visitor(left);
            visitor(right);
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            visitor(condition);
            visitor(then_expr);
            visitor(else_expr);
        }
        ExprKind::FunctionCall { args, .. }
        | ExprKind::StaticMethodCall { args, .. }
        | ExprKind::ClosureCall { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::NewScopedObject { args, .. } => {
            for argument in args {
                visitor(call_arg_value(argument));
            }
        }
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            visitor(object);
            for argument in args {
                visitor(call_arg_value(argument));
            }
        }
        ExprKind::NullsafeDynamicMethodCall {
            object,
            method,
            args,
        } => {
            visitor(object);
            visitor(method);
            for argument in args {
                visitor(call_arg_value(argument));
            }
        }
        ExprKind::NewDynamic { name_expr, args } => {
            visitor(name_expr);
            for argument in args {
                visitor(call_arg_value(argument));
            }
        }
        ExprKind::NewDynamicObject {
            class_name, args, ..
        } => {
            visitor(class_name);
            for argument in args {
                visitor(call_arg_value(argument));
            }
        }
        ExprKind::ExprCall { callee, args } => {
            visitor(callee);
            for argument in args {
                visitor(call_arg_value(argument));
            }
        }
        ExprKind::ArrayLiteral(items) => {
            for item in items {
                visitor(item);
            }
        }
        ExprKind::ArrayLiteralAssoc(entries) => {
            for (key, value) in entries {
                visitor(key);
                visitor(value);
            }
        }
        ExprKind::NamedArg { value, .. }
        | ExprKind::Spread(value)
        | ExprKind::ErrorSuppress(value)
        | ExprKind::Print(value)
        | ExprKind::Negate(value)
        | ExprKind::BitNot(value)
        | ExprKind::Not(value)
        | ExprKind::Clone(value)
        | ExprKind::Throw(value)
        | ExprKind::YieldFrom(value)
        | ExprKind::Cast { expr: value, .. }
        | ExprKind::IncludeValue { path: value, .. }
        | ExprKind::PtrCast { expr: value, .. }
        | ExprKind::BufferNew { len: value, .. } => visitor(value),
        ExprKind::InstanceOf { value, target } => {
            visitor(value);
            if let crate::parser::ast::InstanceOfTarget::Expr(target) = target {
                visitor(target);
            }
        }
        ExprKind::ArrayAccess { array, index } => {
            visitor(array);
            visitor(index);
        }
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. }
        | ExprKind::ObjectClassName { object } => visitor(object),
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            visitor(object);
            visitor(property);
        }
        ExprKind::FirstClassCallable(crate::parser::ast::CallableTarget::Method {
            object,
            ..
        }) => visitor(object),
        ExprKind::Match {
            subject,
            arms,
            default,
        } => {
            visitor(subject);
            for (conditions, value) in arms {
                for condition in conditions {
                    visitor(condition);
                }
                visitor(value);
            }
            if let Some(default) = default {
                visitor(default);
            }
        }
        ExprKind::Assignment {
            target,
            value,
            result_target,
            ..
        } => {
            visitor(target);
            visitor(value);
            if let Some(result_target) = result_target {
                visitor(result_target);
            }
        }
        ExprKind::Yield { key, value } => {
            if let Some(key) = key {
                visitor(key);
            }
            if let Some(value) = value {
                visitor(value);
            }
        }
        _ => {}
    }
}

/// Returns the local array name targeted by `array_push`'s first argument.
fn array_push_target_name(args: &[Expr]) -> Option<&str> {
    let first = args.first()?;
    match &call_arg_value(first).kind {
        ExprKind::Variable(name) => Some(name),
        _ => None,
    }
}

/// Unwraps a named call argument to its value expression.
fn call_arg_value(argument: &Expr) -> &Expr {
    match &argument.kind {
        ExprKind::NamedArg { value, .. } => value,
        _ => argument,
    }
}
