//! Purpose:
//! Lowers by-reference arguments that need an explicit caller-visible read/mutate/write-back
//! sequence through a hidden temporary. This covers non-local container places and gradual
//! local storage whose runtime representation differs from a declared parameter.
//!
//! Called from:
//! - Direct function, instance-method, nullsafe-method, and static-method call lowering before
//!   their ordinary argument materialization.
//!
//! Key details:
//! - Only a receiver the backend can resolve to a slot reaches its COW write-back
//!   (`ReceiverPlace` in `crate::codegen::lower_inst::receiver_place`), and a plain local
//!   variable is the only argument shape that produces one.
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
//! - Declared scalar parameters may use a concrete temporary when their caller local has boxed
//!   gradual storage. The shared gradual-boundary conversion validates or converts the current
//!   payload before the call, and the post-call store publishes the result back into the boxed
//!   caller slot.
//! - Place types are resolved statically (no IR is emitted before the decision), so a shape
//!   this module cannot resolve falls through to the pre-existing lowering unchanged.

use crate::ir_lower::context::{LoweredValue, LoweringContext};
use crate::names::Name;
use crate::parser::ast::{Expr, ExprKind};
use crate::types::{FunctionSig, PhpType};

use super::{
    call_signature, lower_expr, lower_function_call, lower_non_local_assignment_write,
    normalize_value_php_type, source_prefers_extension_builtin, static_property_result_type,
};

/// One by-reference argument rewritten into a hidden temporary.
///
/// `place` is the stabilized target expression written back after the call; `temp` is the
/// hidden local holding the adapted or mutated value while the callee runs.
pub(super) struct RefPlacePlan {
    index: usize,
    place: Expr,
    temp: String,
}

/// Lowers a direct call whose by-reference argument needs a temporary place adapter.
///
/// Returns `None` — leaving the call to the ordinary lowering — unless its signature has a
/// by-reference regular parameter bound either to a statically array-typed non-local place or
/// to gradual local storage that needs a declared concrete representation. On a rewrite the
/// place is read into a hidden temporary, the call is re-lowered against that temporary, and
/// the temporary is written back so the caller's storage observes the mutation.
pub(super) fn lower_ref_place_function_call(
    ctx: &mut LoweringContext<'_, '_>,
    name: &Name,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let canonical = name.as_str();
    let prefer_extension = source_prefers_extension_builtin(canonical);
    let sig = call_signature(ctx, canonical, prefer_extension)?;
    let (call_args, plans) = prepare_ref_place_args(ctx, &sig, args)?;
    let result = lower_function_call(ctx, name, &call_args, expr);
    write_back_ref_place_args(ctx, plans);
    Some(result)
}

/// Replaces supported by-reference places and storage adapters with ordinary hidden locals.
///
/// The returned arguments can use the regular call lowering. Each plan retains the stabilized
/// place and hidden local needed by `write_back_ref_place_args()` after the call completes.
/// Returns `None` when no supported place requires rewriting.
pub(super) fn prepare_ref_place_args(
    ctx: &mut LoweringContext<'_, '_>,
    sig: &FunctionSig,
    args: &[Expr],
) -> Option<(Vec<Expr>, Vec<RefPlacePlan>)> {
    if !sig.ref_params.iter().any(|is_ref| *is_ref) {
        return None;
    }
    let rewrite_indices: Vec<usize> = args
        .iter()
        .enumerate()
        .filter(|(index, arg)| {
            ref_param_binding(sig, *index, arg).is_some_and(|(param_index, place)| {
                is_rewritable_non_local_ref_place(ctx, place)
                    || declared_local_ref_needs_adapter(ctx, sig, param_index, place)
            })
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
        let (param_index, place_arg) = ref_param_binding(sig, index, arg)?;
        let place = stabilize_place(ctx, place_arg);
        let read = lower_expr(ctx, &place);
        let needs_adapter = declared_local_ref_needs_adapter(ctx, sig, param_index, place_arg);
        let value_type = if needs_adapter {
            normalize_value_php_type(sig.params.get(param_index)?.1.codegen_repr())
        } else {
            normalize_value_php_type(ctx.builder.value_php_type(read.value))
        };
        let read = if needs_adapter {
            crate::ir_lower::gradual_coercions::coerce_gradual_value_to_boundary(
                ctx,
                read,
                &value_type,
                Some(place_arg.span),
            )
        } else {
            read
        };
        let temp = ctx.declare_synthetic_php_local(value_type.clone());
        ctx.store_local(&temp, read, value_type, Some(place_arg.span));
        let variable = Expr::new(ExprKind::Variable(temp.clone()), place_arg.span);
        call_args[index] = match &arg.kind {
            ExprKind::NamedArg { name, .. } => Expr::new(
                ExprKind::NamedArg {
                    name: name.clone(),
                    value: Box::new(variable),
                },
                arg.span,
            ),
            _ => variable,
        };
        plans.push(RefPlacePlan { index, place, temp });
    }
    // Every rewritten argument now names a plain local, directly or as a named argument's
    // value, so recursive direct-call lowering cannot trigger this rewrite again.
    debug_assert!(plans.iter().all(|plan| {
        let rewritten = &call_args[plan.index];
        let place = match &rewritten.kind {
            ExprKind::NamedArg { value, .. } => value.as_ref(),
            _ => rewritten,
        };
        matches!(place.kind, ExprKind::Variable(_))
    }));
    Some((call_args, plans))
}

/// Writes every mutated hidden local back to its stabilized caller-visible place.
pub(super) fn write_back_ref_place_args(
    ctx: &mut LoweringContext<'_, '_>,
    plans: Vec<RefPlacePlan>,
) {
    for plan in plans {
        let value_type = ctx.local_type(&plan.temp);
        widen_local_container_place(ctx, &plan.place, value_type.clone());
        let value = Expr::new(ExprKind::Variable(plan.temp), plan.place.span);
        if let ExprKind::Variable(name) = &plan.place.kind {
            let lowered = lower_expr(ctx, &value);
            ctx.store_local(name, lowered, value_type, Some(plan.place.span));
        } else {
            lower_non_local_assignment_write(ctx, &plan.place, &value, plan.place.span);
        }
    }
}

/// Publishes a rewritten element's post-call representation in its local root type.
///
/// A declared bare `array` by-reference parameter widens a concrete `Array(T)` argument to
/// `Array(Mixed)` before the call. When that argument came from `$map[$key]`, future reads of
/// the child must use the widened element layout too; otherwise they interpret boxed Mixed
/// cells as raw scalar slots. Property-rooted chains keep their declared class metadata and are
/// left unchanged here.
fn widen_local_container_place(
    ctx: &mut LoweringContext<'_, '_>,
    place: &Expr,
    value_type: PhpType,
) {
    let ExprKind::ArrayAccess { array, .. } = &place.kind else {
        return;
    };
    if let Some((name, root_type)) = widened_local_container_type(ctx, array, value_type) {
        ctx.set_local_type(&name, root_type);
    }
}

/// Rebuilds the type of a local-rooted container chain after one element changes layout.
fn widened_local_container_type(
    ctx: &LoweringContext<'_, '_>,
    container: &Expr,
    child_type: PhpType,
) -> Option<(String, PhpType)> {
    let container_type = static_place_type(ctx, container)?.codegen_repr();
    let widened = match container_type {
        PhpType::Array(_) => PhpType::Array(Box::new(child_type)),
        PhpType::AssocArray { key, .. } => PhpType::AssocArray {
            key,
            value: Box::new(child_type),
        },
        _ => return None,
    };
    match &container.kind {
        ExprKind::Variable(name) => Some((name.clone(), widened)),
        ExprKind::ArrayAccess { array, .. } => {
            widened_local_container_type(ctx, array, widened)
        }
        _ => None,
    }
}

/// Returns the argument expression bound to a by-reference parameter, or `None`.
///
/// A positional argument binds to the parameter at the same index; a named argument
/// (`sort(array: $obj->items)`) binds to the parameter its name selects, so both call forms
/// reach the same rewrite. Variadic tail positions are excluded because only the visible
/// regular parameters carry the registry's by-reference markers.
fn ref_param_binding<'a>(
    sig: &FunctionSig,
    index: usize,
    arg: &'a Expr,
) -> Option<(usize, &'a Expr)> {
    let regular_param_count = crate::types::call_args::regular_param_count(sig);
    let (param_index, place) = match &arg.kind {
        ExprKind::NamedArg { name, value } => (
            sig.params.iter().position(|(param, _)| param == name)?,
            value.as_ref(),
        ),
        _ => (index, arg),
    };
    if param_index >= regular_param_count {
        return None;
    }
    if !sig.ref_params.get(param_index).copied().unwrap_or(false) {
        return None;
    }
    Some((param_index, place))
}

/// Returns whether a boxed caller local needs a concrete temporary for this declared ref param.
fn declared_local_ref_needs_adapter(
    ctx: &LoweringContext<'_, '_>,
    sig: &FunctionSig,
    param_index: usize,
    place: &Expr,
) -> bool {
    if !sig
        .declared_params
        .get(param_index)
        .copied()
        .unwrap_or(false)
    {
        return false;
    }
    let ExprKind::Variable(name) = &place.kind else {
        return false;
    };
    let Some((_, expected)) = sig.params.get(param_index) else {
        return false;
    };
    let storage = ctx.local_storage_type(name).codegen_repr();
    let expected = expected.codegen_repr();
    matches!(storage, PhpType::Mixed | PhpType::Union(_))
        && !matches!(expected, PhpType::Mixed | PhpType::Union(_))
}

/// Returns whether a by-reference argument is a non-local mutable container place.
///
/// Plain locals are excluded because the existing lowering already writes the separated array
/// back to their frame slot. Gradual property and element values are included because their
/// runtime array shape is intentionally hidden behind `Mixed` until the mutator validates it.
fn is_rewritable_non_local_ref_place(ctx: &LoweringContext<'_, '_>, arg: &Expr) -> bool {
    if !is_candidate_place_shape(arg) {
        return false;
    }
    static_place_type(ctx, arg).is_some_and(|php_type| {
        matches!(
            php_type.codegen_repr(),
            PhpType::Array(_)
                | PhpType::AssocArray { .. }
                | PhpType::Mixed
                | PhpType::Union(_)
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
/// index is evaluated once into a synthetic local and both evaluations read that local. The
/// rest of the receiver chain is composed exclusively of the shapes `static_place_type`
/// resolves, which are side-effect-free local, property, and element reads.
///
/// Infallible by construction: the caller only reaches this for an argument
/// `static_place_type` already resolved, and that resolver matches exactly the variants below.
/// An unmatched shape is returned unchanged, which is the conservative identity — it cannot be
/// reached without emitting IR for a place this module then refuses to write back.
fn stabilize_place(ctx: &mut LoweringContext<'_, '_>, place: &Expr) -> Expr {
    match &place.kind {
        ExprKind::PropertyAccess { object, property } => {
            let object = stabilize_place(ctx, object);
            Expr::new(
                ExprKind::PropertyAccess {
                    object: Box::new(object),
                    property: property.clone(),
                },
                place.span,
            )
        }
        ExprKind::ArrayAccess { array, index } => {
            let array = stabilize_place(ctx, array);
            let index = stabilize_index(ctx, index);
            Expr::new(
                ExprKind::ArrayAccess {
                    array: Box::new(array),
                    index: Box::new(index),
                },
                place.span,
            )
        }
        _ => place.clone(),
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
