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
//! - Only array/hash-typed places are rewritten. Scalar by-reference parameters (`settype`,
//!   `preg_match` `$matches`, `str_replace` `$count`) keep their existing lowering and their
//!   existing diagnostics, because a hidden temp declared with the place's scalar type cannot
//!   represent a builtin that re-types its argument.
//! - Place types are resolved statically (no IR is emitted before the decision), so a shape
//!   this module cannot resolve falls through to the pre-existing lowering unchanged.

use crate::ir_lower::context::{LoweredValue, LoweringContext};
use crate::ir::{ArrayKeySort, Immediate, Op};
use crate::names::{
    php_symbol_key, property_hook_get_method, property_hook_set_method, Name,
};
use crate::parser::ast::{Expr, ExprKind};
use crate::types::{FunctionSig, PhpType};

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
    if args.iter().any(is_spread_arg) {
        // A spread cannot be split into per-parameter places here; PHP also rejects spreading
        // into a by-reference parameter, and the checker already reports that.
        return None;
    }
    let key_sort = match php_symbol_key(canonical.trim_start_matches('\\')).as_str() {
        "ksort" => Some(ArrayKeySort::Ascending),
        "krsort" => Some(ArrayKeySort::Descending),
        _ => None,
    };
    if let Some(sort) = key_sort {
        if sort == ArrayKeySort::Descending {
            if let Some(result) = lower_direct_property_krsort(ctx, canonical, &sig, args, expr) {
                return Some(result);
            }
        }
        if let Some(result) =
            lower_mixed_array_element_key_sort(ctx, canonical, &sig, args, expr, sort)
        {
            return Some(result);
        }
    }
    let rewrite_indices: Vec<usize> = args
        .iter()
        .enumerate()
        .filter(|(index, arg)| {
            ref_param_place(&sig, *index, arg).is_some_and(|place| is_array_place(ctx, place))
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
        let place_arg = ref_param_place(&sig, index, arg)?;
        let place = stabilize_place(ctx, place_arg);
        let read = lower_expr(ctx, &place);
        let value_type = normalize_value_php_type(ctx.builder.value_php_type(read.value));
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
    // Every rewritten argument now names a plain local (directly, or as the value of the named
    // argument it replaced), so the recursive call takes the ordinary by-reference path and
    // this rewrite cannot re-fire.
    debug_assert!(plans.iter().all(|plan| {
        let rewritten = &call_args[plan.index];
        let place = match &rewritten.kind {
            ExprKind::NamedArg { value, .. } => value.as_ref(),
            _ => rewritten,
        };
        matches!(place.kind, ExprKind::Variable(_))
    }));
    let result = lower_function_call(ctx, name, &call_args, expr);
    for plan in plans {
        let value = Expr::new(ExprKind::Variable(plan.temp), plan.place.span);
        lower_non_local_assignment_write(ctx, &plan.place, &value, plan.place.span);
    }
    Some(result)
}

/// Sorts a declared instance-array property through its original object slot.
///
/// The receiver is restricted to a stable local or `$this`, evaluated once, and hooked properties
/// stay on the generic temporary rewrite. The borrowed property payload is retained before a
/// packed-to-hash conversion. Backend receiver-place lowering then republishes both that conversion
/// and a later hash COW split into the same property slot before generic argument cleanup drops the
/// transient owner.
fn lower_direct_property_krsort(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    sig: &FunctionSig,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let [arg] = args else {
        return None;
    };
    let place = ref_param_place(sig, 0, arg)?;
    let ExprKind::PropertyAccess { object, property } = &place.kind else {
        return None;
    };
    if !matches!(object.kind, ExprKind::Variable(_) | ExprKind::This)
        || property_requires_generic_write_context(ctx, object, property)
    {
        return None;
    }
    let property_ty = static_place_type(ctx, place)?.codegen_repr();
    if !matches!(&property_ty, PhpType::Array(_) | PhpType::AssocArray { .. }) {
        return None;
    }

    let object = lower_expr(ctx, object);
    let property_value = super::property_access::lower_property_get_from_value(
        ctx,
        object,
        property,
        Op::PropGet,
        place,
    );
    let property_value =
        crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(place.span));
    let hash = match property_ty {
        PhpType::Array(element_ty) => {
            let assoc_ty = PhpType::AssocArray {
                key: Box::new(PhpType::Int),
                value: element_ty,
            };
            ctx.emit_value(
                Op::ArrayToHash,
                vec![property_value.value],
                None,
                assoc_ty,
                Op::ArrayToHash.default_effects(),
                Some(place.span),
            )
        }
        PhpType::AssocArray { .. } => property_value,
        _ => return None,
    };
    Some(super::emit_builtin_call_value(
        ctx,
        name,
        vec![hash.value],
        PhpType::Bool,
        expr.span,
        None,
    ))
}

/// Reports whether hooks or readonly enforcement require the generic property write path.
fn property_requires_generic_write_context(
    ctx: &LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
) -> bool {
    let Some(class_name) = place_object_class_name(ctx, object) else {
        return true;
    };
    let Some(class_info) = ctx.classes.get(class_name.as_str()) else {
        return true;
    };
    let getter = php_symbol_key(&property_hook_get_method(property));
    let setter = php_symbol_key(&property_hook_set_method(property));
    class_info.readonly_properties.contains(property)
        || class_info.methods.contains_key(&getter)
        || class_info.methods.contains_key(&setter)
}

/// Sorts one nested array cell through an independently mutable boxed-Mixed payload.
///
/// Direct packed or associative parents are first COW-separated and widened. Parents that already
/// store Mixed cells instead clone and republish only the selected cell, preventing shallow parent
/// aliases from observing promotion. The guarded runtime accepts tag 4 (promote), tag 5 (borrow),
/// and raises `TypeError` for every other tag.
fn lower_mixed_array_element_key_sort(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    sig: &FunctionSig,
    args: &[Expr],
    expr: &Expr,
    sort: ArrayKeySort,
) -> Option<LoweredValue> {
    let [arg] = args else {
        return None;
    };
    let place = ref_param_place(sig, 0, arg)?;
    let ExprKind::ArrayAccess { array, index } = &place.kind else {
        return None;
    };
    let ExprKind::Variable(parent_name) = &array.kind else {
        return None;
    };
    if let PhpType::Array(element_ty) = ctx.local_type(parent_name).codegen_repr() {
        if sort == ArrayKeySort::Ascending && element_ty.codegen_repr() != PhpType::Mixed {
            return None;
        }
        return lower_mixed_packed_array_element_key_sort(
            ctx,
            name,
            parent_name,
            array,
            index,
            expr,
            *element_ty,
            sort,
        );
    }
    let PhpType::AssocArray { key, value } = ctx.local_type(parent_name).codegen_repr() else {
        return None;
    };
    let value_repr = value.codegen_repr();
    if sort == ArrayKeySort::Ascending && value_repr != PhpType::Mixed {
        return None;
    }
    if !matches!(
        &value_repr,
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Mixed
    ) {
        return None;
    }

    let mixed_parent_ty = PhpType::AssocArray {
        key,
        value: Box::new(PhpType::Mixed),
    };
    if value_repr != PhpType::Mixed {
        let parent = ctx.load_local(parent_name, Some(array.span));
        ctx.prepare_mutated_local_owner(
            parent_name,
            parent,
            mixed_parent_ty.clone(),
            Some(array.span),
        );
        let mixed_parent = ctx.emit_value(
            Op::HashToMixed,
            vec![parent.value],
            None,
            mixed_parent_ty.clone(),
            Op::HashToMixed.default_effects(),
            Some(array.span),
        );
        ctx.store_prepared_mutated_local(
            parent_name,
            mixed_parent,
            mixed_parent_ty,
            Some(array.span),
        );
    }

    let parent = ctx.load_local(parent_name, Some(array.span));
    let key = lower_expr(ctx, index);
    let cell_op = if value_repr == PhpType::Mixed {
        Op::HashGet
    } else {
        Op::HashGetForWrite
    };
    let cell = ctx.emit_value(
        cell_op,
        vec![parent.value, key.value],
        None,
        PhpType::Mixed,
        cell_op.default_effects(),
        Some(expr.span),
    );
    if value_repr == PhpType::Mixed {
        return lower_shared_mixed_hash_element_key_sort(
            ctx, name, parent, key, cell, expr, sort,
        );
    }
    lower_attached_mixed_cell_key_sort(ctx, name, cell, expr, sort)
}

/// Sorts one packed-parent Mixed cell, widening the parent only when it is still concrete.
///
/// The first nested sort converts `array<array<T>>` to stored Mixed cells after separating the
/// parent for copy-on-write. Later sibling sorts reuse the resulting `array<mixed>` directly so
/// `ArrayToMixed` never receives an already-Mixed input and the guarded cell promotion remains the
/// sole authority for accepting a packed child, borrowing a promoted hash, or raising `TypeError`.
fn lower_mixed_packed_array_element_key_sort(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    parent_name: &str,
    array: &Expr,
    index: &Expr,
    expr: &Expr,
    element_ty: PhpType,
    sort: ArrayKeySort,
) -> Option<LoweredValue> {
    let element_repr = element_ty.codegen_repr();
    if !matches!(&element_repr, PhpType::Array(_) | PhpType::Mixed)
        || super::index_expr_key_type(ctx, index) != PhpType::Int
    {
        return None;
    }
    let mixed_parent_ty = PhpType::Array(Box::new(PhpType::Mixed));
    if element_repr != PhpType::Mixed {
        let parent = ctx.load_local(parent_name, Some(array.span));
        ctx.prepare_mutated_local_owner(
            parent_name,
            parent,
            mixed_parent_ty.clone(),
            Some(array.span),
        );
        let mixed_parent = ctx.emit_value(
            Op::ArrayToMixed,
            vec![parent.value],
            None,
            mixed_parent_ty.clone(),
            Op::ArrayToMixed.default_effects(),
            Some(array.span),
        );
        ctx.store_prepared_mutated_local(
            parent_name,
            mixed_parent,
            mixed_parent_ty,
            Some(array.span),
        );
    }

    let parent = ctx.load_local(parent_name, Some(array.span));
    let key = lower_expr(ctx, index);
    let key = super::coerce_to_int_at_span(ctx, key, Some(index.span));
    let cell_op = if element_repr == PhpType::Mixed {
        Op::ArrayGet
    } else {
        Op::ArrayGetForWrite
    };
    let cell = ctx.emit_value(
        cell_op,
        vec![parent.value, key.value],
        None,
        PhpType::Mixed,
        cell_op.default_effects(),
        Some(expr.span),
    );
    if element_repr == PhpType::Mixed {
        return lower_shared_mixed_array_element_key_sort(
            ctx, name, parent, key, cell, expr, sort,
        );
    }
    lower_attached_mixed_cell_key_sort(ctx, name, cell, expr, sort)
}

/// Detaches one shared associative-parent cell before publishing and sorting its promoted hash.
///
/// Parent COW is performed by `HashSet`; cloning first prevents a shallow parent split from
/// exposing an in-place cell promotion through aliases. Failed promotion occurs before insertion,
/// so missing or scalar elements keep the guarded `TypeError` path without autovivification.
fn lower_shared_mixed_hash_element_key_sort(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    parent: LoweredValue,
    key: LoweredValue,
    cell: LoweredValue,
    expr: &Expr,
    sort: ArrayKeySort,
) -> Option<LoweredValue> {
    let cloned = clone_mixed_cell(ctx, cell, expr);
    let hash = promote_mixed_cell_to_hash(ctx, cloned, expr, sort);
    ctx.emit_void(
        Op::HashSet,
        vec![parent.value, key.value, cloned.value],
        None,
        Op::HashSet.default_effects(),
        Some(expr.span),
    );
    let result = super::emit_builtin_call_value(
        ctx,
        name,
        vec![hash.value],
        PhpType::Bool,
        expr.span,
        None,
    );
    crate::ir_lower::ownership::release_if_owned(ctx, cloned, Some(expr.span));
    Some(result)
}

/// Detaches one shared packed-parent cell before publishing and sorting its promoted hash.
///
/// `ArraySet` performs the parent COW split only after guarded promotion succeeds, preserving the
/// absent/scalar failure behavior while installing an independently owned cell for mutation.
fn lower_shared_mixed_array_element_key_sort(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    parent: LoweredValue,
    key: LoweredValue,
    cell: LoweredValue,
    expr: &Expr,
    sort: ArrayKeySort,
) -> Option<LoweredValue> {
    let cloned = clone_mixed_cell(ctx, cell, expr);
    let hash = promote_mixed_cell_to_hash(ctx, cloned, expr, sort);
    ctx.emit_void(
        Op::ArraySet,
        vec![parent.value, key.value, cloned.value],
        None,
        Op::ArraySet.default_effects(),
        Some(expr.span),
    );
    let result = super::emit_builtin_call_value(
        ctx,
        name,
        vec![hash.value],
        PhpType::Bool,
        expr.span,
        None,
    );
    crate::ir_lower::ownership::release_if_owned(ctx, cloned, Some(expr.span));
    Some(result)
}

/// Clones a stored Mixed cell and releases the borrowed/owned source handle.
fn clone_mixed_cell(
    ctx: &mut LoweringContext<'_, '_>,
    cell: LoweredValue,
    expr: &Expr,
) -> LoweredValue {
    let cloned = ctx.emit_owned_value(
        Op::RuntimeCall,
        vec![cell.value],
        Some(Immediate::RuntimeCall(
            crate::ir::RuntimeCallTarget::MixedCellClone,
        )),
        PhpType::Mixed,
        super::effects_lookup::runtime_effects(),
        Some(expr.span),
    );
    crate::ir_lower::ownership::release_if_owned(ctx, cell, Some(expr.span));
    cloned
}

/// Promotes a guarded Mixed cell to the borrowed hash representation consumed by a key sort.
fn promote_mixed_cell_to_hash(
    ctx: &mut LoweringContext<'_, '_>,
    cell: LoweredValue,
    expr: &Expr,
    sort: ArrayKeySort,
) -> LoweredValue {
    ctx.emit_value(
        Op::RuntimeCall,
        vec![cell.value],
        Some(Immediate::RuntimeCall(
            crate::ir::RuntimeCallTarget::MixedCellPromoteToHash(sort),
        )),
        PhpType::AssocArray {
            key: Box::new(PhpType::Int),
            value: Box::new(PhpType::Mixed),
        },
        super::effects_lookup::runtime_effects(),
        Some(expr.span),
    )
}

/// Promotes a write-fetched cell and marks the returned hash as attached to its parent place.
fn promote_attached_mixed_cell_to_hash(
    ctx: &mut LoweringContext<'_, '_>,
    cell: LoweredValue,
    expr: &Expr,
    sort: ArrayKeySort,
) -> LoweredValue {
    ctx.emit_value(
        Op::RuntimeCall,
        vec![cell.value],
        Some(Immediate::RuntimeCall(
            crate::ir::RuntimeCallTarget::MixedCellPromoteAttachedToHash(sort),
        )),
        PhpType::AssocArray {
            key: Box::new(PhpType::Int),
            value: Box::new(PhpType::Mixed),
        },
        super::effects_lookup::runtime_effects(),
        Some(expr.span),
    )
}

/// Promotes and sorts a cell fetched for write after its concrete parent was widened.
fn lower_attached_mixed_cell_key_sort(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    cell: LoweredValue,
    expr: &Expr,
    sort: ArrayKeySort,
) -> Option<LoweredValue> {
    let hash = promote_attached_mixed_cell_to_hash(ctx, cell, expr, sort);
    let result = super::emit_builtin_call_value(
        ctx,
        name,
        vec![hash.value],
        PhpType::Bool,
        expr.span,
        None,
    );
    crate::ir_lower::ownership::release_if_owned(ctx, cell, Some(expr.span));
    Some(result)
}

/// Returns the argument expression bound to a by-reference parameter, or `None`.
///
/// A positional argument binds to the parameter at the same index; a named argument
/// (`sort(array: $obj->items)`) binds to the parameter its name selects, so both call forms
/// reach the same rewrite. Variadic tail positions are excluded because only the visible
/// regular parameters carry the registry's by-reference markers.
fn ref_param_place<'a>(sig: &FunctionSig, index: usize, arg: &'a Expr) -> Option<&'a Expr> {
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
    Some(place)
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
