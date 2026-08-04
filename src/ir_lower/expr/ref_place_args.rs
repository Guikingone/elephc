//! Purpose:
//! Lowers a mutating builtin call whose by-reference array argument is a *place* other than
//! a plain local — an object property, a static property, or a container element — as an
//! explicit read/mutate/write-back sequence through a hidden temporary.
//!
//! Called from:
//! - `crate::ir_lower::expr::lower_function_call()`, before the builtin fast paths, so the
//!   rewritten call re-enters the ordinary local-variable by-reference lowering.
//!
//! Key details:
//! - Only the local-variable form of a by-reference builtin argument reaches the backend's
//!   COW write-back (`source_load_local_slot` in `crate::codegen::lower_inst::builtins::arrays`).
//!   A property or element operand is loaded, `acquire`d, handed to the runtime, and released,
//!   so `__rt_array_ensure_unique` separates a private copy that nothing ever stores back.
//!   That is a silent wrong answer: `usort($obj->items, ...)` used to leave `$obj->items`
//!   untouched with no diagnostic.
//! - The rewrite is `$tmp = <place>; f($tmp, ...); <place> = $tmp;`, where `$tmp` is a
//!   synthetic slot with ordinary PHP local ownership (`declare_synthetic_php_local`), so the
//!   store retains even when the place read is a borrowed pointer — a static-property load
//!   carries no reference of its own, and moving it into a hidden temp would let the
//!   write-back's release of the previous occupant free an array another variable still holds.
//! - Because `$tmp` retains, the runtime's ensure-unique sees a shared buffer and separates
//!   before mutating — which is exactly PHP's copy-on-write behavior: an earlier
//!   `$c = $obj->items;` alias stays unsorted, and a `usort` comparator that reads the
//!   property while sorting still sees the pre-sort array.
//! - Only array/hash-typed places are rewritten. Scalar by-reference parameters (`settype`,
//!   `preg_match` `$matches`, `str_replace` `$count`) keep their existing lowering and their
//!   existing diagnostics, because a hidden temp declared with the place's scalar type cannot
//!   represent a builtin that re-types its argument.
//! - Place types are resolved statically (no IR is emitted before the decision), so a shape
//!   this module cannot resolve falls through to the pre-existing lowering unchanged.

use crate::ir_lower::context::{LoweredValue, LoweringContext};
use crate::names::Name;
use crate::parser::ast::{Expr, ExprKind};
use crate::types::PhpType;

use super::{
    call_signature, is_spread_arg, lower_expr, lower_function_call,
    lower_non_local_assignment_write, normalize_value_php_type, source_prefers_extension_builtin,
    static_property_result_type,
};

/// One by-reference argument rewritten into a hidden temporary.
///
/// `place` is the stabilized target expression written back after the call; `temp` is the
/// hidden local holding the mutated array while the builtin runs.
struct RefPlacePlan {
    index: usize,
    place: Expr,
    temp: String,
}

/// Lowers a builtin call whose by-reference array argument is a non-local place.
///
/// Returns `None` — leaving the call to the ordinary lowering — unless the callee is a
/// registry builtin with a by-reference regular parameter and at least one such argument is a
/// statically array-typed property, static property, or container element. On a rewrite the
/// place is read into a hidden temporary, the call is re-lowered against that temporary, and
/// the temporary is written back to the place so the caller's storage observes the mutation.
pub(super) fn lower_builtin_ref_place_call(
    ctx: &mut LoweringContext<'_, '_>,
    name: &Name,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let canonical = name.as_str();
    let prefer_extension = source_prefers_extension_builtin(canonical);
    if !prefer_extension
        && (ctx.functions.contains_key(canonical) || ctx.extern_functions.contains_key(canonical))
    {
        // User-defined and extern callees own a separate by-reference machine that already
        // rejects non-local arguments with a named diagnostic.
        return None;
    }
    let sig = call_signature(ctx, canonical, prefer_extension)?;
    if !sig.ref_params.iter().any(|is_ref| *is_ref) {
        return None;
    }
    if crate::types::call_args::has_named_args(args) || args.iter().any(is_spread_arg) {
        return None;
    }
    let regular_param_count = crate::types::call_args::regular_param_count(&sig);
    let rewrite_indices: Vec<usize> = args
        .iter()
        .enumerate()
        .take(regular_param_count)
        .filter(|(index, arg)| {
            sig.ref_params.get(*index).copied().unwrap_or(false) && is_array_place(ctx, arg)
        })
        .map(|(index, _)| index)
        .collect();
    if rewrite_indices.is_empty() {
        return None;
    }
    let mut call_args: Vec<Expr> = args.to_vec();
    let mut plans: Vec<RefPlacePlan> = Vec::with_capacity(rewrite_indices.len());
    for index in rewrite_indices {
        let arg = &args[index];
        let place = stabilize_place(ctx, arg)?;
        let read = lower_expr(ctx, &place);
        let value_type = normalize_value_php_type(ctx.builder.value_php_type(read.value));
        let temp = ctx.declare_synthetic_php_local(value_type.clone());
        ctx.store_local(&temp, read, value_type, Some(arg.span));
        call_args[index] = Expr::new(ExprKind::Variable(temp.clone()), arg.span);
        plans.push(RefPlacePlan { index, place, temp });
    }
    // Every rewritten argument is now a plain local, so the recursive call takes the ordinary
    // by-reference path and this rewrite cannot re-fire.
    debug_assert!(plans
        .iter()
        .all(|plan| matches!(call_args[plan.index].kind, ExprKind::Variable(_))));
    let result = lower_function_call(ctx, name, &call_args, expr);
    for plan in plans {
        let value = Expr::new(ExprKind::Variable(plan.temp), plan.place.span);
        lower_non_local_assignment_write(ctx, &plan.place, &value, plan.place.span);
    }
    Some(result)
}

/// Returns whether a by-reference argument is a non-local place holding array storage.
///
/// Plain locals are excluded because the existing lowering already writes the separated array
/// back to their frame slot. Scalar places are excluded so builtins that re-type their
/// by-reference argument keep their current lowering and diagnostics.
fn is_array_place(ctx: &LoweringContext<'_, '_>, arg: &Expr) -> bool {
    if !is_candidate_place_shape(arg) {
        return false;
    }
    static_place_type(ctx, arg).is_some_and(|php_type| {
        matches!(
            php_type.codegen_repr(),
            PhpType::Array(_) | PhpType::AssocArray { .. }
        )
    })
}

/// Returns whether an argument has one of the place shapes this rewrite can read and write.
fn is_candidate_place_shape(arg: &Expr) -> bool {
    matches!(
        arg.kind,
        ExprKind::PropertyAccess { .. }
            | ExprKind::StaticPropertyAccess { .. }
            | ExprKind::ArrayAccess { .. }
    )
}

/// Resolves the static PHP type of a place expression without emitting any IR.
///
/// Only the shapes this module can read and write back are resolved — locals, `$this`,
/// declared instance properties, declared static properties, and elements of those. Anything
/// else returns `None`, which keeps the call on its pre-existing lowering path.
fn static_place_type(ctx: &LoweringContext<'_, '_>, expr: &Expr) -> Option<PhpType> {
    match &expr.kind {
        ExprKind::Variable(name) => {
            if ctx.has_local_slot(name) {
                Some(ctx.local_type(name))
            } else {
                None
            }
        }
        ExprKind::This => {
            if ctx.has_local_slot("this") {
                Some(ctx.local_type("this"))
            } else {
                None
            }
        }
        ExprKind::PropertyAccess { object, property } => {
            let class_name = place_object_class_name(ctx, object)?;
            let class_info = ctx.classes.get(class_name.as_str())?;
            let (_, (_, property_ty)) = class_info.visible_property(property)?;
            Some(normalize_value_php_type(property_ty.clone()))
        }
        ExprKind::StaticPropertyAccess { receiver, property } => Some(
            static_property_result_type(ctx, receiver, property, expr),
        ),
        ExprKind::ArrayAccess { array, .. } => {
            match static_place_type(ctx, array)?.codegen_repr() {
                PhpType::Array(elem_ty) => Some(normalize_value_php_type(*elem_ty)),
                PhpType::AssocArray { value, .. } => Some(normalize_value_php_type(*value)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Resolves the class a property receiver refers to, for property-type lookup.
///
/// Returns `None` for a receiver whose static type is not a single known class — `Mixed`,
/// a union, or an unresolved local — so the caller leaves the argument on its existing path.
fn place_object_class_name(ctx: &LoweringContext<'_, '_>, object: &Expr) -> Option<String> {
    match static_place_type(ctx, object)?.codegen_repr() {
        PhpType::Object(class_name) => Some(class_name.trim_start_matches('\\').to_string()),
        _ => None,
    }
}

/// Rebuilds a place expression so it can be evaluated twice — once to read, once to write.
///
/// Container indexes are the only sub-expression that may carry side effects, so a non-trivial
/// index is evaluated once into a hidden temporary and both evaluations read that temporary.
/// The receiver chain is composed exclusively of the place shapes `static_place_type` resolves,
/// which are side-effect-free property and element reads.
fn stabilize_place(ctx: &mut LoweringContext<'_, '_>, place: &Expr) -> Option<Expr> {
    match &place.kind {
        ExprKind::Variable(_) | ExprKind::This | ExprKind::StaticPropertyAccess { .. } => {
            Some(place.clone())
        }
        ExprKind::PropertyAccess { object, property } => {
            let object = stabilize_place(ctx, object)?;
            Some(Expr::new(
                ExprKind::PropertyAccess {
                    object: Box::new(object),
                    property: property.clone(),
                },
                place.span,
            ))
        }
        ExprKind::ArrayAccess { array, index } => {
            let array = stabilize_place(ctx, array)?;
            let index = stabilize_index(ctx, index);
            Some(Expr::new(
                ExprKind::ArrayAccess {
                    array: Box::new(array),
                    index: Box::new(index),
                },
                place.span,
            ))
        }
        _ => None,
    }
}

/// Evaluates a container index once when re-evaluating it could repeat a side effect.
///
/// Literals and already-stored locals are re-read directly; anything else is lowered into a
/// synthetic local whose variable reference replaces the original index expression, so
/// `sort($m[next_index()])` calls `next_index()` exactly once like PHP.
fn stabilize_index(ctx: &mut LoweringContext<'_, '_>, index: &Expr) -> Expr {
    if matches!(
        index.kind,
        ExprKind::Variable(_)
            | ExprKind::This
            | ExprKind::IntLiteral(_)
            | ExprKind::FloatLiteral(_)
            | ExprKind::StringLiteral(_)
            | ExprKind::BoolLiteral(_)
            | ExprKind::Null
    ) {
        return index.clone();
    }
    let value = lower_expr(ctx, index);
    let value_type = normalize_value_php_type(ctx.builder.value_php_type(value.value));
    let temp = ctx.declare_synthetic_php_local(value_type.clone());
    ctx.store_local(&temp, value, value_type, Some(index.span));
    Expr::new(ExprKind::Variable(temp), index.span)
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Unit coverage for the by-reference place rewrite's argument classification.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - These assertions are pure predicates over AST shapes; the end-to-end behavior is
    //!   covered by `tests/codegen/arrays/` and `tests/codegen/objects/property_access/`.

    use super::*;
    use crate::parser::ast::StaticReceiver;
    use crate::span::Span;

    /// A plain local argument is never treated as a rewritable place: the backend's
    /// local-slot write-back already stores the separated array back for it.
    #[test]
    fn plain_local_is_not_a_candidate_place_shape() {
        let local = Expr::new(ExprKind::Variable("a".to_string()), Span::dummy());
        assert!(!is_candidate_place_shape(&local));
    }

    /// Property, static-property, and element arguments are the shapes the rewrite considers.
    #[test]
    fn property_and_element_shapes_are_candidate_places() {
        let span = Span::dummy();
        let object = Expr::new(ExprKind::Variable("o".to_string()), span);
        let property = Expr::new(
            ExprKind::PropertyAccess {
                object: Box::new(object.clone()),
                property: "items".to_string(),
            },
            span,
        );
        let static_property = Expr::new(
            ExprKind::StaticPropertyAccess {
                receiver: StaticReceiver::Self_,
                property: "items".to_string(),
            },
            span,
        );
        let element = Expr::new(
            ExprKind::ArrayAccess {
                array: Box::new(object),
                index: Box::new(Expr::new(ExprKind::IntLiteral(0), span)),
            },
            span,
        );
        assert!(is_candidate_place_shape(&property));
        assert!(is_candidate_place_shape(&static_property));
        assert!(is_candidate_place_shape(&element));
    }
}
