//! Purpose:
//! Direct array writes, key promotion, and write-operand ownership.
//!
//! Called from:
//! - `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Preserves statement ordering, CFG shape, EIR effects, and ownership contracts.

use super::*;

/// Releases the value operand of an array/hash element write when it is an owned
/// string. These writes PERSIST (copy) a string value into the container instead
/// of moving it (`__rt_str_persist`), so an owned string operand — e.g. a function
/// or extern call result like `$_ENV[$k] = getenv_value()` — would otherwise never
/// be freed (a per-write heap leak that exhausts the heap under `--web`). Non-string
/// refcounted values (objects, arrays) are moved, or retained only when borrowed,
/// by the write itself, so they must not be released here.
pub(in crate::ir_lower) fn release_persisted_string_operand(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) {
    let ty = ctx.builder.value_php_type(value.value);
    // Only release a FRESH owning string temporary (a call/concat result, etc.).
    // A borrowed load of a variable that still owns the string (e.g. the prelude's
    // `$_GET[$k] = $v`) must NOT be released here, or the container's stored copy
    // would be freed out from under it.
    if matches!(ty.codegen_repr(), PhpType::Str) && ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
}

/// Releases a string written into a Mixed container slot when the slot's box COPIED it.
///
/// A box takes over a heap string that a call or `StrPersist` produced
/// (`Op::yields_box_absorbable_string`), so that one is no longer this write's to release --
/// releasing it freed the string the container now holds. Every other fresh string -- a
/// `(string)` cast, a concatenation -- is copied into the box and still has to be dropped.
pub(in crate::ir_lower) fn release_string_copied_into_box(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) {
    let absorbed = ctx
        .builder
        .value_defining_op(value.value)
        .is_some_and(Op::yields_box_absorbable_string);
    if !absorbed {
        release_persisted_string_operand(ctx, value, span);
    }
}

/// Releases an indexed-array write operand when the backend retained or copied it.
pub(in crate::ir_lower) fn release_indexed_array_write_operand(
    ctx: &mut LoweringContext<'_, '_>,
    container_elem_ty: Option<&PhpType>,
    value: LoweredValue,
    span: Span,
) {
    if !ctx.value_is_owning_temporary(value) {
        return;
    }
    let value_ty = ctx.builder.value_php_type(value.value).codegen_repr();
    if matches!(
        container_elem_ty.map(PhpType::codegen_repr),
        Some(PhpType::Mixed)
    ) && !matches!(value_ty, PhpType::Mixed | PhpType::Union(_))
    {
        return;
    }
    crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
}

/// Returns the indexed-array element type in effect for a write.
pub(in crate::ir_lower) fn indexed_array_write_element_type(
    ctx: &LoweringContext<'_, '_>,
    array_value: LoweredValue,
    updated_ty: Option<&PhpType>,
) -> Option<PhpType> {
    let array_ty = updated_ty
        .cloned()
        .unwrap_or_else(|| ctx.builder.value_php_type(array_value.value));
    match array_ty.codegen_repr() {
        PhpType::Array(elem_ty) => Some(elem_ty.codegen_repr()),
        _ => None,
    }
}

/// Lowers the key and the value of an element write in PHP's evaluation order.
///
/// PHP freezes an index *expression* into a temporary before the right-hand side runs, so
/// `$a[idx()] = val()` prints `[idx][val]` and stores at whatever `idx()` returned. But a
/// plain *variable* index is not frozen: the store reads the variable's slot at store time,
/// after the right-hand side, so `$i = 0; $a[$i] = ($i = 1);` writes index 1, not index 0.
/// Only the read moves — the index expression keeps its place so side-effect order is
/// unchanged, which for a bare variable is no order at all.
///
/// The value goes through `lower_array_reference_or_value`, so `$a[$k] = &$x` materializes the
/// source's ref cell instead of a plain read. That path is a strict extension: an expression that
/// is not an `ArrayReference` marker lowers exactly as `lower_expr` would.
pub(super) fn lower_write_key_and_value(
    ctx: &mut LoweringContext<'_, '_>,
    index: &Expr,
    value: &Expr,
) -> (LoweredValue, LoweredValue) {
    if matches!(index.kind, ExprKind::Variable(_)) {
        let value_value = lower_array_reference_or_value(ctx, value);
        return (lower_expr(ctx, index), value_value);
    }
    // A scratch key (`"svc$i"`) must outlive a value whose calls reset the concat buffer:
    // `$defs["svc$i"] = (new Def())->m([new Ref("a$i")])->m(["name$i"])` keyed the entry "name"
    // once the second concatenation reused the bytes, so Symfony's container lost services.
    let index_value = lower_expr(ctx, index);
    let index_value = crate::ir_lower::expr::persist_concat_lhs_if_rhs_can_reset(
        ctx,
        index_value,
        value,
        index.span,
    );
    (index_value, lower_array_reference_or_value(ctx, value))
}

/// Returns whether `expr` holds a node whose lowering must not be emitted twice: a closure or an
/// anonymous class registers a function/class body per lowered occurrence.
fn offset_operand_is_duplicable(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::IntLiteral(_)
        | ExprKind::StringLiteral(_)
        | ExprKind::BoolLiteral(_)
        | ExprKind::Null
        | ExprKind::Variable(_) => true,
        ExprKind::PreIncrement(_)
        | ExprKind::PreDecrement(_)
        | ExprKind::PostIncrement(_)
        | ExprKind::PostDecrement(_) => true,
        ExprKind::BinaryOp { left, right, .. } => {
            offset_operand_is_duplicable(left) && offset_operand_is_duplicable(right)
        }
        ExprKind::ArrayAccess { array, index } => {
            offset_operand_is_duplicable(array) && offset_operand_is_duplicable(index)
        }
        ExprKind::Negate(inner) | ExprKind::Not(inner) => offset_operand_is_duplicable(inner),
        ExprKind::Cast { expr, .. } => offset_operand_is_duplicable(expr),
        ExprKind::FunctionCall { args, .. } => args.iter().all(offset_operand_is_duplicable),
        ExprKind::Ternary { condition, then_expr, else_expr } => {
            offset_operand_is_duplicable(condition)
                && offset_operand_is_duplicable(then_expr)
                && offset_operand_is_duplicable(else_expr)
        }
        _ => false,
    }
}

/// Lowers `$container[$index] = $value` for a local whose storage is a boxed `Mixed`/union cell by
/// dispatching on the cell's run-time tag, and returns whether it did.
///
/// php decides between a string offset write and an element write when the statement runs. The
/// element write has a lowering for every array shape, but a boxed container that holds a STRING
/// reached no lowering at all: the store was dropped, so `function f($s) { $s[0] = 'H'; return
/// $s; }` handed the string back unchanged (symfony/polyfill-mbstring's `mb_convert_case()` then
/// upper-cased nothing). The statement becomes
/// `if (is_string($c)) { $t = (string) $c; $t[$i] = $v; $c = $t; } else { <the original write> }`:
/// the string arm borrows the `string`-typed offset write, the other arm is the write this
/// lowering always emitted. Only one arm runs, so the operands are evaluated once either way; they
/// are lowered twice, so operands that register code per occurrence (closures) opt out.
fn lower_boxed_container_offset_write(
    ctx: &mut LoweringContext<'_, '_>,
    array: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
) -> bool {
    if ctx.offset_dispatch_bypass.contains(&span) {
        return false;
    }
    if crate::superglobals::is_superglobal(array)
        || crate::globals_array::is_alias(array)
        || !ctx.has_local_slot(array)
        || !matches!(
            ctx.local_type(array).codegen_repr(),
            PhpType::Mixed | PhpType::Union(_)
        )
        || !offset_operand_is_duplicable(index)
        || !offset_operand_is_duplicable(value)
    {
        return false;
    }
    use crate::synthetic_class::{e_call, e_cast, e_var, s_assign, s_if};
    let scratch = "__elephc_offset_string";
    let original = Stmt::new(
        StmtKind::ArrayAssign {
            array: array.to_string(),
            index: index.clone(),
            value: value.clone(),
        },
        span,
    );
    let string_arm = vec![
        s_assign(scratch, e_cast(crate::parser::ast::CastType::String, e_var(array))),
        Stmt::new(
            StmtKind::ArrayAssign {
                array: scratch.to_string(),
                index: index.clone(),
                value: value.clone(),
            },
            span,
        ),
        s_assign(array, e_var(scratch)),
    ];
    let dispatch = s_if(
        e_call("is_string", vec![e_var(array)]),
        string_arm,
        Vec::new(),
        Some(vec![original]),
    );
    // The span is bypassed for the whole synthesized `if`: the string arm's own write targets a
    // `string` local, which never reaches the boxed-container case, so only the array arm's copy
    // of the original statement is affected.
    let before = ctx.local_type(array);
    ctx.offset_dispatch_bypass.insert(span);
    lower_stmt(ctx, &dispatch);
    ctx.offset_dispatch_bypass.remove(&span);
    ctx.set_local_logical_type(array, before);
    true
}

/// Lowers an indexed array assignment.
pub(super) fn lower_array_assign(
    ctx: &mut LoweringContext<'_, '_>,
    array: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
) {
    if lower_boxed_container_offset_write(ctx, array, index, value, span) {
        return;
    }
    let update = desugared_element_update(value, span, |read| {
        reads_local_array_element(read, array, index)
    });
    if let Some(ElementUpdate::NullCoalesce { read, default }) = update {
        crate::ir_lower::expr::lower_null_coalesce_update_stmt(ctx, read, default, span);
        return;
    }
    lower_array_assign_with_diagnosed_key(ctx, array, index, value, span, update.is_some());
}

/// How a statement value produced by the parser's update desugaring re-reads its own target.
///
/// `$place[$k] op= $v` is written as `$place[$k] op $v` and `$place[$k] ??= $v` as
/// `$place[$k] ?? $v`, both built with the statement's own span. A right-hand side the user
/// wrote never carries that span, so `$a[$k] = $a[$k] + 1` stays two separate PHP accesses.
pub(crate) enum ElementUpdate<'a> {
    /// A compound operator. The value already read the element, so the write reuses the key
    /// that read converted instead of diagnosing a float key a second time.
    Compound,
    /// A null-coalescing assignment. `read` probes the element, and `default` is written only
    /// when that probe produced null, as PHP does.
    NullCoalesce { read: &'a Expr, default: &'a Expr },
}

/// Classifies a statement value that re-reads the element the statement writes.
pub(crate) fn desugared_element_update<'a>(
    value: &'a Expr,
    assignment_span: Span,
    reads_target: impl Fn(&Expr) -> bool,
) -> Option<ElementUpdate<'a>> {
    if value.span != assignment_span {
        return None;
    }
    match &value.kind {
        ExprKind::BinaryOp { left, .. } if reads_target(left) => Some(ElementUpdate::Compound),
        ExprKind::NullCoalesce { value: read, default } if reads_target(read) => {
            Some(ElementUpdate::NullCoalesce { read, default })
        }
        _ => None,
    }
}

/// Returns whether `read` is the element `$array[index]` of the named local.
fn reads_local_array_element(read: &Expr, array: &str, index: &Expr) -> bool {
    matches!(
        &read.kind,
        ExprKind::ArrayAccess { array: receiver, index: read_index }
            if matches!(&receiver.kind, ExprKind::Variable(name) if name == array)
                && read_index.as_ref() == index
    )
}

/// Lowers `$string[$offset] = $char` as a string-offset write, storing the result back.
fn lower_string_offset_write(
    ctx: &mut LoweringContext<'_, '_>,
    array: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
) {
    let (mut index_value, mut value_value) = lower_write_key_and_value(ctx, index, value);
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
    let array_value = ctx.load_local(array, Some(span));
    index_value = coerce_to_int_at_span(ctx, index_value, Some(index.span));
    value_value = coerce_to_string_at_span(ctx, value_value, Some(value.span));
    let result = ctx.emit_value(
        Op::StrSetOffset,
        vec![array_value.value, index_value.value, value_value.value],
        None,
        PhpType::Str,
        Op::StrSetOffset.default_effects(),
        Some(span),
    );
    ctx.store_local(array, result, PhpType::Str, Some(span));
    release_persisted_string_operand(ctx, value_value, span);
}

/// Lowers an array write after an earlier read of the same source-level dimension.
pub(crate) fn lower_array_assign_with_diagnosed_key(
    ctx: &mut LoweringContext<'_, '_>,
    array: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
    key_already_diagnosed: bool,
) {
    // The write path is chosen from the container's type BEFORE the key and the value are
    // lowered, and an `eval()` or runtime include among them widens every local to `mixed` as it
    // runs. Widen first, so the path chosen is the one the container has when it is written:
    // `$routes[4] = $forDump ? $code : eval($code)` (Symfony's CompiledUrlMatcherDumper) picked
    // the indexed writer and then handed it the boxed container.
    // A closed-world build only widens for an eval of a literal fragment: a computed string
    // raises instead of running, so it can change no local.
    let widens = |expr: &Expr| {
        crate::ir_lower::expr::expr_contains_eval_call(expr)
            && !(crate::closed_world::is_enabled() && !contains_literal_eval(expr))
    };
    if widens(value) || widens(index) {
        ctx.apply_eval_barrier();
    }
    if matches!(ctx.local_type(array).codegen_repr(), PhpType::Str) {
        lower_string_offset_write(ctx, array, index, value, span);
        return;
    }
    // The path is chosen from the representation the container would load with, without loading
    // it: the container is fetched below, after the key and the value. A real load taken here
    // was left behind once the chosen path loaded again, and a load that unboxes a Mixed-storage
    // slot holds its own reference -- `$set = []; $set['k'] = true;` leaked the array per write
    // in any function whose frame widened to boxed slots.
    // `$boxed[$k] = &$x` binds a reference, which the boxed writer cannot carry: the general
    // path below materializes it through `lower_array_reference_or_value`.
    if matches!(
        ctx.peek_local_load_ir_type(array),
        IrType::Heap(crate::ir::IrHeapKind::Mixed | crate::ir::IrHeapKind::Union)
    ) && !matches!(value.kind, ExprKind::ArrayReference(_))
    {
        lower_boxed_array_local_set(ctx, array, index, value, span, key_already_diagnosed);
        return;
    }
    let (mut index_value, mut value_value) = lower_write_key_and_value(ctx, index, value);
    // The container is fetched for writing AFTER the key and the value, as PHP does: a value that
    // grows the array (`$a['x'] = grow($a)` with a by-reference `grow`) reallocates it, and a
    // pointer loaded first would write into the old block.
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
    let array_value = load_array_local_for_write(ctx, array, span);
    let op = array_set_op(array_value.ir_type);
    // A literal string index always means a hash key, so promote the destination
    // to associative storage like PHP. A boxed Mixed/Union index may hold either
    // an integer or a string key (foreach loop keys are always Mixed in EIR via
    // `Op::IterCurrentKey`), so it goes through `Op::ArraySetMixedKey`, whose
    // runtime helper keeps integer keys on indexed storage (preserving indexed
    // consumers like `implode`) and promotes only string keys to a hash. This
    // stops a `foreach($arr as $k=>$v) $dst[$k]=$v` rebuild from collapsing a
    // string key onto int 0. A foreach key over a concretely-indexed array is
    // known to be int-valued, so it is left on the coerce path to avoid
    // needlessly dispatching.
    if op == Op::ArraySet && index_value.ir_type == IrType::Str {
        lower_string_key_array_promotion(ctx, array, array_value, index_value, value_value, span);
        return;
    }
    if op == Op::ArraySet
        && index_is_boxed_mixed_key(index_value.ir_type)
        && !index_is_foreach_int_key(ctx, index)
    {
        lower_mixed_key_array_set(
            ctx, array, array_value, index_value, value_value, span, key_already_diagnosed,
        );
        return;
    }
    if op == Op::ArraySet
        && index_value.ir_type == IrType::I64
        && index_requires_runtime_array_key_dispatch(index)
    {
        let index_value = ctx.box_value_as_mixed(index_value, PhpType::Mixed, Some(index.span));
        lower_mixed_key_array_set(
            ctx, array, array_value, index_value, value_value, span, key_already_diagnosed,
        );
        return;
    }
    if op == Op::ArraySet {
        index_value = coerce_array_key_to_int_at_span(
            ctx, index_value, Some(index.span), key_already_diagnosed,
        );
        // A reference marker is the slot's new IDENTITY, not a value to narrow into the current
        // element type: casting it to `int` stored the referenced value's copy and dropped the
        // reference. Left Mixed, the write below widens the container to boxed slots instead.
        if !matches!(value.kind, ExprKind::ArrayReference(_)) {
            let array_ty = ctx.builder.value_php_type(array_value.value);
            value_value =
                coerce_indexed_array_set_value(ctx, &array_ty, value_value, Some(value.span));
        }
    }
    if op == Op::BufferSet {
        index_value = coerce_to_int_at_span(ctx, index_value, Some(index.span));
        let buffer_ty = ctx.builder.value_php_type(array_value.value);
        value_value = coerce_buffer_set_value(ctx, &buffer_ty, value_value, Some(value.span));
    }
    if op == Op::HashSet {
        let current_ty = ctx.builder.value_php_type(array_value.value);
        // A `static` hash keeps one storage type across calls, so this write cannot widen it the
        // way it widens an ordinary local: `$s['k'] = $s['k'] + 1` produced a boxed sum, the
        // slot was rewritten as `array<string, mixed>`, and the next call still read `int`
        // values -- the box's address. Coerce into the slot's int element instead, as the
        // indexed write does for every local.
        if matches!(ctx.local_kinds.get(array), Some(LocalKind::StaticLocal)) {
            value_value = coerce_static_hash_set_value(ctx, &current_ty, value_value, Some(value.span));
        }
        let value_ty = ctx.builder.value_php_type(value_value.value);
        if let Some(updated_ty) = assoc_array_write_updated_type(current_ty, value_ty) {
            ctx.prepare_mutated_local_owner(array, array_value, updated_ty.clone(), Some(span));
            let converted = ctx.emit_value(
                Op::HashToMixed,
                vec![array_value.value],
                None,
                updated_ty.clone(),
                Op::HashToMixed.default_effects(),
                Some(span),
            );
            ctx.emit_void(
                Op::HashSet,
                vec![converted.value, index_value.value, value_value.value],
                None,
                Op::HashSet.default_effects(),
                Some(span),
            );
            release_persisted_string_operand(ctx, index_value, span);
            release_persisted_string_operand(ctx, value_value, span);
            ctx.store_prepared_mutated_local(array, converted, updated_ty, Some(span));
            return;
        }
    }
    if op == Op::ArraySet {
        let (array_value, updated_ty, needs_storeback) =
            prepare_indexed_array_local_set(ctx, array_value, value_value, span);
        ctx.emit_void(
            op,
            vec![array_value.value, index_value.value, value_value.value],
            None,
            op.default_effects(),
            Some(span),
        );
        let elem_ty = indexed_array_write_element_type(ctx, array_value, updated_ty.as_ref());
        finish_indexed_array_local_write(
            ctx,
            array,
            array_value,
            updated_ty,
            needs_storeback,
            span,
        );
        release_indexed_array_write_operand(ctx, elem_ty.as_ref(), value_value, span);
        return;
    }
    ctx.emit_void(
        op,
        vec![array_value.value, index_value.value, value_value.value],
        (op == Op::HashSet && key_already_diagnosed).then_some(Immediate::Bool(true)),
        op.default_effects(),
        Some(span),
    );
    release_persisted_string_operand(ctx, index_value, span);
    if op == Op::RuntimeCall {
        release_object_offset_set_operand(ctx, value_value, span);
        // The key is borrowed by `offsetSet()` too. An object key unboxed from a slot another
        // store widened to boxed storage (`$s[$q] = 1` with `$q` later reassigned) is an owned
        // unbox, and `$s[new P()] = 1` builds one; neither was released.
        if !matches!(ctx.builder.value_php_type(index_value.value).codegen_repr(), PhpType::Str) {
            release_object_offset_set_operand(ctx, index_value, span);
        }
    } else {
        release_persisted_string_operand(ctx, value_value, span);
    }
}

/// Releases the value an `ArrayAccess` write handed to `offsetSet()`.
///
/// `$o[$k] = v` on an object is a method call: `offsetSet()` BORROWS its arguments and acquires
/// what it stores, so a fresh temporary of any kind is still this write's to drop. Releasing
/// only strings leaked one array per `$o[$k] = [ ... ]` -- every `SplObjectStorage`/`WeakMap`
/// entry and every user `ArrayAccess` write of a literal.
pub(in crate::ir_lower) fn release_object_offset_set_operand(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) {
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
}

/// Returns whether an integer-keyed indexed write needs a runtime storage dispatch.
///
/// Dense indexed storage can represent only known non-negative literal offsets. A dynamic
/// integer may be negative or sparse, both of which require hash storage in PHP; boxing it and
/// routing through `ArraySetMixedKey` keeps the dense fast path for proven literals while the
/// runtime helper promotes the other cases without losing the key.
fn index_requires_runtime_array_key_dispatch(index: &Expr) -> bool {
    !matches!(index.kind, ExprKind::IntLiteral(value) if value >= 0)
}

/// Returns the widened associative-array type needed by a heterogeneous local write.
///
/// A concrete hash value representation cannot describe a later value of another PHP type.
/// Widening to `Mixed` forces `HashToMixed` to preserve the earlier entries' runtime tags before
/// the new entry is inserted, so later gradual reads and callback boundaries see each real type.
pub(super) fn assoc_array_write_updated_type(current_ty: PhpType, value_ty: PhpType) -> Option<PhpType> {
    let PhpType::AssocArray { key, value } = current_ty.codegen_repr() else {
        return None;
    };
    if hash_storage_represents_written_value(&value.codegen_repr(), &value_ty.codegen_repr()) {
        return None;
    }
    Some(PhpType::AssocArray {
        key,
        value: Box::new(PhpType::Mixed),
    })
}

/// Returns true when a hash stamped with `storage` can hold a `written` value as it stands.
///
/// The question is about the RUNTIME REPRESENTATION, not about PHP assignability: an entry is
/// stored as a raw payload plus the tag `storage` dictates, and every later read materializes it
/// through `storage` as well. So the only writes that need no widening are the ones whose payload
/// already has the stored shape.
///
/// `normalize_array_write_element_type` cannot answer this: it maps every refcounted non-string
/// type to `Mixed`, which conflates a hash whose entries are raw object pointers with one whose
/// entries are boxed Mixed cells. Under that conflation an `array<string, N>` looked like it
/// already accepted anything -- `$nodes = ['node' => $node]; $nodes['n'] = 1;` compiled and then
/// segfaulted reading an integer as an object, and `twig/twig`'s
/// `TestExpression::__construct` -- which writes a `?Node` parameter into exactly such a literal --
/// stopped the backend with `hash_set value PHP type Mixed`.
fn hash_storage_represents_written_value(storage: &PhpType, written: &PhpType) -> bool {
    // Boxed storage describes every payload: each entry carries its own tag.
    if matches!(storage, PhpType::Mixed | PhpType::Iterable) {
        return true;
    }
    if storage == written {
        return true;
    }
    // An object payload is a pointer whose class is read from the object itself -- property and
    // method access dispatch on the runtime class id -- so two classes share one representation.
    // A container payload does NOT: its ELEMENT type is what a read of `$hash[k][i]` materializes,
    // so `['x' => [1, 2]]` followed by `$a['y'] = ['s', 't']` must widen. Letting the two share a
    // slot printed the element's raw pointer as an integer.
    matches!((storage, written), (PhpType::Object(_), PhpType::Object(_)))
}

/// Roots and retires operands borrowed by the boxed writer, including on same-frame catches.
/// The backend copies strings and retains other payloads in a separate consumed box.
/// `key_already_diagnosed` marks the write half of a compound update, whose read already
/// reported a float key's conversion, so the boxed writer rebuilds the key silently.
fn lower_boxed_array_local_set(
    ctx: &mut LoweringContext<'_, '_>,
    array: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
    key_already_diagnosed: bool,
) {
    let mut roots = Vec::new();
    let mut lower_operand = |ctx: &mut LoweringContext<'_, '_>, expr: &Expr| {
        let value = lower_expr(ctx, expr);
        let (value, root) = crate::ir_lower::expr::root_owned_call_operand(ctx, value, expr.span);
        if let Some(root) = root { roots.push(root); }
        value
    };
    // Preserve the existing PHP ordering for bare variable keys versus computed keys.
    let (index, value) = if matches!(index.kind, ExprKind::Variable(_)) {
        let value = lower_operand(ctx, value);
        (lower_operand(ctx, index), value)
    } else {
        let index = lower_operand(ctx, index);
        (index, lower_operand(ctx, value))
    };
    // The box is detached and fetched AFTER the operands, like the typed paths: a by-reference
    // call among them (`$v['r'] = rec($v)`) detaches and republishes the shared cell's box, and a
    // box fetched first received the write and was then dropped -- every such entry vanished.
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
    // Detaching stores a clone into the slot and releases the box the slot held. An operand
    // BORROWED from that box -- the local itself in `$context['_parent'] = $context`, or one of
    // its elements -- would then be read after its owner was freed: every compiled Twig template
    // starts its loops with that exact statement on a bare `array $context` parameter, and the
    // Symfony --web worker died on `/` with a bad refcount. A lifetime pin keeps each borrowed
    // operand alive across the detach and the write.
    let mut pins = Vec::new();
    for operand in [index, value] {
        if ctx.value_is_owning_temporary(operand) {
            continue;
        }
        let pin = crate::ir_lower::ownership::acquire_lifetime_pin_if_refcounted(ctx, operand, Some(span));
        if pin.value != operand.value {
            pins.push(pin);
        }
    }
    let array = load_array_local_for_write(ctx, array, span);
    ctx.emit_void(
        Op::RuntimeCall,
        vec![array.value, index.value, value.value],
        key_already_diagnosed.then_some(Immediate::Bool(true)),
        Op::RuntimeCall.default_effects(),
        Some(span),
    );
    for pin in pins.into_iter().rev() {
        crate::ir_lower::ownership::release_if_owned(ctx, pin, Some(span));
    }
    for root in roots.into_iter().rev() {
        crate::ir_lower::expr::retire_owned_call_operand(ctx, root, span);
    }
}

/// Coerces a buffer element write value into the scalar storage accepted by `BufferSet`.
pub(super) fn coerce_buffer_set_value(
    ctx: &mut LoweringContext<'_, '_>,
    buffer_ty: &PhpType,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    let coerced = match buffer_ty.codegen_repr() {
        PhpType::Buffer(elem_ty) => match elem_ty.codegen_repr() {
            PhpType::Float => coerce_to_float(ctx, value, span),
            PhpType::Int | PhpType::Bool => coerce_to_int(ctx, value, span),
            _ => value,
        },
        _ => value,
    };
    if coerced.value != value.value && ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, span);
    }
    coerced
}

/// Promotes an indexed local array to a Mixed-valued associative array for string-key writes.
pub(super) fn lower_string_key_array_promotion(
    ctx: &mut LoweringContext<'_, '_>,
    array: &str,
    array_value: LoweredValue,
    index: LoweredValue,
    value: LoweredValue,
    span: Span,
) {
    let current_ty = ctx.builder.value_php_type(array_value.value);
    let value_ty = ctx.builder.value_php_type(value.value);
    let assoc_ty = promoted_assoc_array_type(current_ty, value_ty);
    ctx.prepare_mutated_local_owner_for_backend_retire(array, array_value, assoc_ty.clone(), Some(span));
    // The slot's reference moves into the conversion; codegen consumes only a marked move.
    let hash = ctx.emit_local_array_to_hash_move(array, array_value, assoc_ty.clone(), span);
    ctx.emit_void(
        Op::HashSet,
        vec![hash.value, index.value, value.value],
        None,
        Op::HashSet.default_effects(),
        Some(span),
    );
    release_persisted_string_operand(ctx, index, span);
    release_persisted_string_operand(ctx, value, span);
    ctx.store_prepared_mutated_local(array, hash, assoc_ty, Some(span));
}

/// Writes `value` into the indexed local `array` under a boxed Mixed/Union key.
///
/// The destination stays statically `Array(Mixed)` (so indexed consumers such as
/// `implode` keep routing to the indexed path) while `Op::ArraySetMixedKey`
/// dispatches the key tag at runtime: integer keys stay on indexed storage and
/// string keys promote the destination to a hash. This is the Mixed-key analogue
/// of `lower_string_key_array_promotion`, which unconditionally promotes because
/// a literal string key is always a hash key.
pub(super) fn lower_mixed_key_array_set(
    ctx: &mut LoweringContext<'_, '_>,
    array: &str,
    array_value: LoweredValue,
    index: LoweredValue,
    value: LoweredValue,
    span: Span,
    key_already_diagnosed: bool,
) {
    let mixed_array_ty = PhpType::Array(Box::new(PhpType::Mixed));
    // The helper consumes the slot's reference and hands back the one the slot will own -- the
    // same array written in place, a grown or split copy, or the hash it promoted to -- so this is
    // the consuming-mutation protocol `lower_string_key_array_promotion` uses: a boxed owner is
    // dropped before the write, and the result is stored without releasing the old pointer, which
    // the growth may already have freed or the split left to its other owner.
    ctx.prepare_mutated_local_owner(array, array_value, mixed_array_ty.clone(), Some(span));
    let result = ctx.emit_value(
        Op::ArraySetMixedKey,
        vec![array_value.value, index.value, value.value],
        key_already_diagnosed.then_some(Immediate::Bool(true)),
        mixed_array_ty.clone(),
        Op::ArraySetMixedKey.default_effects(),
        Some(span),
    );
    ctx.store_prepared_mutated_local(array, result, mixed_array_ty, Some(span));
    // The boxed key is only read by the helper, so a key boxed for this write is dropped here.
    if ctx.value_is_owning_temporary(index) {
        crate::ir_lower::ownership::release_if_owned(ctx, index, Some(span));
    }
    // A boxed value is retained by the store for the slot, so a boxed temporary -- an element just
    // read with `$values[$i]` -- keeps its own reference too and is dropped here. Without it
    // `$chunk[$k] = $values[$k]` in a loop that had widened `$k` leaked one box per iteration.
    if matches!(
        ctx.builder.value_php_type(value.value).codegen_repr(),
        PhpType::Mixed | PhpType::Union(_)
    ) && ctx.value_is_owning_temporary(value)
    {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
    // A fresh string the box COPIED -- `$out[$key] = (string) $value` -- is still this write's
    // to drop; without it every such element leaked the cast's string. One the box took over (a
    // call result) is not.
    release_string_copied_into_box(ctx, value, span);
}

/// Returns the associative type produced by a string-key write to an indexed array.
pub(super) fn promoted_assoc_array_type(current_ty: PhpType, value_ty: PhpType) -> PhpType {
    let value_ty = normalize_array_write_element_type(value_ty.codegen_repr());
    let assoc_value_ty = match current_ty.codegen_repr() {
        PhpType::Array(elem_ty) if is_empty_indexed_array_element(elem_ty.as_ref()) => value_ty,
        PhpType::Array(elem_ty) => {
            let elem_ty = normalize_array_write_element_type(elem_ty.codegen_repr());
            if elem_ty == value_ty {
                elem_ty
            } else {
                PhpType::Mixed
            }
        }
        _ => PhpType::Mixed,
    };
    PhpType::AssocArray {
        key: Box::new(PhpType::Mixed),
        value: Box::new(assoc_value_ty),
    }
}

/// Whether `expr` contains an `eval()` whose code is a string literal (the only eval a
/// closed-world build still runs); every other eval there raises instead of running code.
fn contains_literal_eval(expr: &Expr) -> bool {
    let text = format!("{expr:?}");
    let mut rest = text.as_str();
    while let Some(at) = rest.find("\"eval\"") {
        let after = &rest[at..];
        let window_end = after.find("FunctionCall").unwrap_or(after.len()).min(240);
        if after[..window_end].contains("args: [Expr { kind: StringLiteral(") {
            return true;
        }
        rest = &after[5..];
    }
    false
}
