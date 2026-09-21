//! Purpose:
//! Array mutation and comparator-specific builtin argument lowering.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers `array_push($local, $value)` as a direct indexed-array mutation.
pub(super) fn lower_static_array_push(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if php_symbol_key(name.trim_start_matches('\\')) != "array_push" || args.len() != 2 {
        return None;
    }
    if crate::types::call_args::has_named_args(args) || args.iter().any(is_spread_arg) {
        return None;
    }
    let ExprKind::Variable(array_name) = &args[0].kind else {
        return None;
    };
    if !matches!(ctx.local_type(array_name).codegen_repr(), PhpType::Array(_)) {
        return None;
    }
    let array_value = ctx.load_local(array_name, Some(args[0].span));
    if array_value.ir_type != IrType::Heap(IrHeapKind::Array) {
        return None;
    }
    let value = lower_expr(ctx, &args[1]);
    let (array_value, updated_ty, needs_storeback) =
        if crate::ir_lower::stmt::ref_bound_mixed_indexed_array_write(ctx, array_name, value) {
            (array_value, Some(ctx.local_type(array_name)), true)
        } else {
            crate::ir_lower::stmt::prepare_indexed_array_local_write(ctx, array_value, value, expr.span)
        };
    ctx.emit_void(
        Op::ArrayPush,
        vec![array_value.value, value.value],
        None,
        Op::ArrayPush.default_effects(),
        Some(expr.span),
    );
    let elem_ty =
        crate::ir_lower::stmt::indexed_array_write_element_type(ctx, array_value, updated_ty.as_ref());
    crate::ir_lower::stmt::finish_indexed_array_local_write(
        ctx,
        array_name,
        array_value,
        updated_ty,
        needs_storeback,
        expr.span,
    );
    crate::ir_lower::stmt::release_indexed_array_write_operand(ctx, elem_ty.as_ref(), value, expr.span);
    // PHP answers the array's new element count. Reload first: an append that grew the container
    // reallocated it, and the write-back above is what published the pointer that is now live.
    let grown = ctx.load_local(array_name, Some(args[0].span));
    Some(ctx.emit_value(
        Op::ArrayLen,
        vec![grown.value],
        None,
        PhpType::Int,
        Op::ArrayLen.default_effects(),
        Some(expr.span),
    ))
}

/// Promotes an indexed-array local to hash storage before a sort that KEEPS its keys.
///
/// `asort([3, 1, 2])` answers keys `1|2|0` in PHP: the key travels with the value, which is the
/// entire difference from `sort()`. An indexed array has no key storage to carry, so the slot
/// permuter the backend uses for a list renumbered them — the values came out ordered and the keys
/// came out `0|1|2`, a silent wrong answer. The result is only representable as a hash.
///
/// This is the same promotion `unset($list[$k])` performs at its own site
/// (`lower_unset_indexed_element`), and the codegen already routes a hash receiver to
/// `__rt_hash_asort` / `__rt_hash_arsort`, so nothing downstream changes.
///
/// `uasort` has the same fault and is deliberately absent: no `__rt_hash_uasort` exists, so
/// promoting it would replace a wrong answer with a refusal to compile.
fn promote_indexed_receiver_for_key_preserving_sort(
    ctx: &mut LoweringContext<'_, '_>,
    canonical: &str,
    args: &[Expr],
) {
    if !matches!(canonical, "asort" | "arsort") {
        return;
    }
    let Some(receiver) = args.first() else {
        return;
    };
    let ExprKind::Variable(name) = &receiver.kind else {
        return;
    };
    let PhpType::Array(element) = ctx.local_type(name).codegen_repr() else {
        return;
    };
    let array_value = ctx.load_local(name, Some(receiver.span));
    let assoc_ty = PhpType::AssocArray {
        key: Box::new(PhpType::Int),
        value: Box::new(element.codegen_repr()),
    };
    let hash = ctx.emit_value(
        Op::ArrayToHash,
        vec![array_value.value],
        None,
        assoc_ty.clone(),
        Op::ArrayToHash.default_effects(),
        Some(receiver.span),
    );
    ctx.store_mutated_local(name, hash, assoc_ty, Some(receiver.span));
}

/// Lowers builtin call operands, applying builtin-specific preservation where source order matters.
pub(super) fn lower_builtin_call_args(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    if is_empty_static_indexed_spread_arg(args) && zero_arity_call_signature(name, sig) {
        return Vec::new();
    }
    let canonical = php_symbol_key(name.trim_start_matches('\\'));
    if canonical == "eval" {
        return lower_eval_args(ctx, sig, args);
    }
    prepare_regex_match_output_local(ctx, &canonical, args);
    promote_indexed_receiver_for_key_preserving_sort(ctx, &canonical, args);
    let pcntl_outputs = prepare_pcntl_output_locals(ctx, &canonical, sig, args);
    let argument_lowering = crate::builtins::registry::lookup(&canonical)
        .map(|def| def.spec.semantics.argument_lowering)
        .unwrap_or(crate::builtins::semantics::BuiltinArgumentLowering::Standard);
    let lowered = match argument_lowering {
        crate::builtins::semantics::BuiltinArgumentLowering::Count => {
            lower_count_args(ctx, sig, args)
        }
        crate::builtins::semantics::BuiltinArgumentLowering::Date => {
            lower_date_args(ctx, sig, args)
        }
        crate::builtins::semantics::BuiltinArgumentLowering::JsonDecode => {
            lower_json_decode_args(ctx, sig, args)
        }
        crate::builtins::semantics::BuiltinArgumentLowering::Getenv => {
            lower_getenv_args(ctx, sig, args)
        }
        crate::builtins::semantics::BuiltinArgumentLowering::PcntlPreserveOmitted => {
            lower_args_with_signature_trimming_trailing_defaults(ctx, sig, args)
        }
        crate::builtins::semantics::BuiltinArgumentLowering::PregReplaceCallback
            if !crate::types::call_args::has_named_args(args)
                && !args.iter().any(is_spread_arg) =>
        {
            lower_preg_replace_callback_args(ctx, sig, args)
        }
        crate::builtins::semantics::BuiltinArgumentLowering::PositionalRegex
            if !crate::types::call_args::has_named_args(args)
                && !args.iter().any(is_spread_arg) =>
        {
            lower_positional_builtin_args_with_signature(ctx, sig, args)
        }
        crate::builtins::semantics::BuiltinArgumentLowering::UserValueSort
            if !crate::types::call_args::has_named_args(args)
                && !args.iter().any(is_spread_arg) =>
        {
            lower_user_value_sort_args(ctx, sig, args)
        }
        crate::builtins::semantics::BuiltinArgumentLowering::ReverseKeySort => {
            lower_reverse_key_sort_args(ctx, sig, args)
        }
        crate::builtins::semantics::BuiltinArgumentLowering::OpensslEncrypt => {
            prepare_openssl_encrypt_tag_local(ctx, args);
            if !crate::types::call_args::has_named_args(args)
                && !args.iter().any(is_spread_arg)
            {
                lower_positional_builtin_args_with_signature(ctx, sig, args)
            } else {
                lower_args_with_signature(ctx, sig, args)
            }
        }
        crate::builtins::semantics::BuiltinArgumentLowering::ArraySplice
            if !args.iter().any(is_spread_arg) =>
        {
            lower_array_splice_args(ctx, sig, args)
        }
        crate::builtins::semantics::BuiltinArgumentLowering::XmlHandlerSetter => {
            lower_xml_handler_setter_call(ctx, &canonical, sig, args)
        }
        _ if canonical == "krsort" && !args.iter().any(is_spread_arg) => {
            lower_key_sort_args(ctx, sig, args)
        }
        _ if canonical == "array_unshift"
            && !crate::types::call_args::has_named_args(args)
            && !args.iter().any(is_spread_arg) =>
        {
            lower_array_unshift_args(ctx, sig, args)
        }
        _ if matches!(canonical.as_str(), "array_keys" | "array_values")
            && !crate::types::call_args::has_named_args(args)
            && !args.iter().any(is_spread_arg) =>
        {
            let mut operands = lower_args_with_signature(ctx, sig, args);
            if let Some(&array) = operands.first() {
                if matches!(
                    ctx.builder.value_php_type(array).codegen_repr(),
                    PhpType::Mixed | PhpType::Union(_)
                ) {
                    operands[0] = convert_mixed_array_to_assoc_hash(ctx, array, args[0].span);
                }
            }
            operands
        }
        _ if !crate::types::call_args::has_named_args(args)
            && !args.iter().any(is_spread_arg) =>
        {
            lower_positional_builtin_args_with_signature(ctx, sig, args)
        }
        _ => lower_args_with_signature(ctx, sig, args),
    };
    for (name, ty) in pcntl_outputs {
        ctx.set_local_logical_type(&name, ty);
    }
    lowered
}

/// Widens PCNTL output storage before its write-only by-reference loads are lowered.
fn prepare_pcntl_output_locals(
    ctx: &mut LoweringContext<'_, '_>,
    canonical: &str,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<(String, PhpType)> {
    let mut outputs = Vec::new();
    if !crate::types::call_args::has_named_args(args) && !args.iter().any(is_spread_arg) {
        for (index, arg) in args.iter().enumerate() {
            if let Some(output) = prepare_pcntl_output_local(ctx, canonical, index, arg) {
                outputs.push(output);
            }
        }
        return outputs;
    }
    let Some(sig) = sig else {
        return outputs;
    };
    let call_span = args
        .first()
        .map(|arg| arg.span)
        .unwrap_or_else(crate::span::Span::dummy);
    let regular_param_count = crate::types::call_args::regular_param_count(sig);
    let Ok(plan) = crate::types::call_args::plan_call_args_with_regular_param_count_and_assoc_spreads(
        sig,
        args,
        call_span,
        regular_param_count,
        false,
        true,
        &assoc_spread_sources(ctx, args),
    ) else {
        return outputs;
    };
    for (index, arg) in plan.regular_args.iter().enumerate() {
        let crate::types::call_args::PlannedRegularArg::Source { expr, .. } = arg else {
            continue;
        };
        if let Some(output) = prepare_pcntl_output_local(ctx, canonical, index, expr) {
            outputs.push(output);
        }
    }
    outputs
}

/// Widens a `preg_match`/`preg_match_all` `$matches` destination that cannot hold a match array.
///
/// `$m = []; preg_match($re, $s, $m);` types the local from the empty literal — `array<never>`,
/// an element type no written value satisfies — and the by-reference write never widened the
/// frame storage, so the backend refused the destination outright:
/// `unsupported EIR backend feature: preg_match matches destination PHP type Array(Never)`.
/// Symfony's `UrlMatcher::matchCollection` opens `$hostMatches = []` exactly like that, which is
/// what kept the routing component out of the compiled world.
///
/// Only `array<never>` is touched. Any other destination already describes storage the runtime
/// can fill, and `set_local_type` would REPLACE the logical type rather than join it, so widening
/// a `Mixed` destination here would narrow it.
///
/// The shape comes from [`crate::types::checker::regex_matches_destination_type`], the same
/// function the checker types the destination with — deciding it again here is how an indexed
/// destination ends up holding a named-capture hash and reads back renumbered.
fn prepare_regex_match_output_local(
    ctx: &mut LoweringContext<'_, '_>,
    canonical: &str,
    args: &[Expr],
) {
    if canonical != "preg_match" && canonical != "preg_match_all" {
        return;
    }
    let Some(Expr {
        kind: ExprKind::Variable(name),
        ..
    }) = args.get(2)
    else {
        return;
    };
    // The RAW local type, not `codegen_repr()`: the representation mapping erases `Never` into
    // the element type an array can actually hold, so the empty-literal destination this exists
    // for reads back as an ordinary array and the guard never fires.
    if !matches!(ctx.local_type(name), PhpType::Array(element) if matches!(*element, PhpType::Never))
    {
        return;
    }
    let widened = crate::types::checker::regex_matches_destination_type(canonical, args);
    ctx.set_local_type(name, widened);
}

/// Widens one direct PCNTL output slot without reinterpreting its pre-call value.
fn prepare_pcntl_output_local(
    ctx: &mut LoweringContext<'_, '_>,
    canonical: &str,
    parameter_index: usize,
    value: &Expr,
) -> Option<(String, PhpType)> {
    let ty = match (canonical, parameter_index) {
        ("pcntl_wait", 0) | ("pcntl_waitpid", 1) => PhpType::Int,
        ("pcntl_wait", 2) | ("pcntl_waitpid", 3) => PhpType::AssocArray {
            key: Box::new(PhpType::Str),
            value: Box::new(PhpType::Int),
        },
        ("pcntl_waitid", 2) => PhpType::AssocArray {
            key: Box::new(PhpType::Str),
            value: Box::new(PhpType::Mixed),
        },
        ("pcntl_waitid", 4) => PhpType::AssocArray {
            key: Box::new(PhpType::Str),
            value: Box::new(PhpType::Int),
        },
        ("pcntl_sigprocmask", 2) => PhpType::Array(Box::new(PhpType::Int)),
        ("pcntl_sigwaitinfo", 1) | ("pcntl_sigtimedwait", 1) => PhpType::AssocArray {
            key: Box::new(PhpType::Str),
            value: Box::new(PhpType::Mixed),
        },
        _ => return None,
    };
    let ExprKind::Variable(name) = &value.kind else {
        return None;
    };
    if ctx.local_type(name).codegen_repr() != ty.codegen_repr() {
        ctx.set_local_type(name, PhpType::Mixed);
    }
    Some((name.clone(), ty))
}

/// Lowers `array_unshift()` operands and widens an incompatible local payload to boxed Mixed.
///
/// Empty arrays carry a `Void`/`Never` placeholder element type. Prepending a gradual value into
/// that raw scalar layout would store a boxed-cell pointer while the array header still describes
/// null slots. Convert the caller-visible local first so both the header and subsequent reads use
/// `Array(Mixed)`. Concrete values that already fit the payload keep the compact typed path.
fn lower_array_unshift_args(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    let mut operands = lower_positional_builtin_args_with_signature(ctx, sig, args);
    let Some(sig) = sig else {
        return operands;
    };
    let Some((name, span)) = first_parameter_plain_local(ctx, sig, args) else {
        return operands;
    };
    let PhpType::Array(element) = ctx.local_type(&name).codegen_repr() else {
        return operands;
    };
    let element = element.codegen_repr();
    let values_already_fit = operands.iter().skip(1).all(|value| {
            let value = ctx.builder.value_php_type(*value).codegen_repr();
            value == element
                || (matches!(element, PhpType::Void | PhpType::Never)
                    && matches!(value, PhpType::Int | PhpType::Bool))
        });
    // `array<mixed>` is a semantic label, not proof that an empty property/default array has
    // already been stamped with boxed-Mixed slots at runtime. Normalize it before prepending so a
    // gradual value cannot be stored as a raw pointer in the empty array's scalar layout. The
    // runtime helper checks tag 7 and is a no-op for arrays that are already normalized.
    if element != PhpType::Mixed && values_already_fit {
        return operands;
    }
    let target = PhpType::Array(Box::new(PhpType::Mixed));
    let local = ctx.load_local(&name, Some(span));
    let converted = ctx.emit_value(
        Op::ArrayToMixed,
        vec![local.value],
        None,
        target.clone(),
        Op::ArrayToMixed.default_effects(),
        Some(span),
    );
    ctx.store_call_normalized_local(&name, converted, target, Some(span));
    if let Some(receiver) = operands.first_mut() {
        *receiver = ctx.load_local(&name, Some(span)).value;
    }
    operands
}

/// Lowers key-sort operands and promotes a plain indexed receiver to integer-keyed hash storage.
fn lower_key_sort_args(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    let mut operands = if crate::types::call_args::has_named_args(args) {
        lower_args_with_signature(ctx, sig, args)
    } else {
        lower_positional_builtin_args_with_signature(ctx, sig, args)
    };
    let Some((name, span)) = key_sort_receiver_local(ctx, sig, args) else {
        return operands;
    };
    let local_type = ctx.local_type(&name).codegen_repr();
    let PhpType::Array(value_type) = local_type else {
        return operands;
    };
    if matches!(value_type.codegen_repr(), PhpType::Never | PhpType::Void) {
        return operands;
    }

    let hash_type = PhpType::AssocArray {
        key: Box::new(PhpType::Int),
        value: value_type,
    };
    let local = ctx.load_local(&name, Some(span));
    let converted = ctx.emit_value(
        Op::ArrayToHash,
        vec![local.value],
        None,
        hash_type.clone(),
        Op::ArrayToHash.default_effects(),
        Some(span),
    );
    ctx.store_mutated_local(&name, converted, hash_type, Some(span));
    if let Some(receiver) = operands.first_mut() {
        *receiver = ctx.load_local(&name, Some(span)).value;
    }
    operands
}

/// Returns the local mutated by a key sort, including aliases and write-back temporaries.
///
/// General argument normalization excludes reference-bound and `__eir_place*` locals from
/// representation changes. Key sorting is different: descending packed keys have no valid
/// in-place representation, so every local receiver must publish the promoted hash. A reference
/// cell propagates that new pointer directly; a ref-place plan writes it back to its container.
fn key_sort_receiver_local(
    ctx: &LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Option<(String, Span)> {
    let receiver = args.iter().enumerate().find_map(|(index, arg)| {
        let (param_index, place) = match &arg.kind {
            ExprKind::NamedArg { name, value } => (
                sig?.params.iter().position(|(param, _)| param == name)?,
                value.as_ref(),
            ),
            _ => (index, arg),
        };
        (param_index == 0).then_some(place)
    })?;
    let ExprKind::Variable(name) = &receiver.kind else {
        return None;
    };
    ctx.has_local_slot(name)
        .then(|| (name.clone(), receiver.span))
}

/// Converts a boxed gradual array operand into an independently owned Mixed-valued hash.
fn convert_mixed_array_to_assoc_hash(
    ctx: &mut LoweringContext<'_, '_>,
    value: crate::ir::ValueId,
    span: Span,
) -> crate::ir::ValueId {
    let source = LoweredValue {
        value,
        ir_type: value_ir_type(&ctx.builder.value_php_type(value)),
    };
    let converted = ctx
        .emit_value(
            Op::MixedToHash,
            vec![value],
            None,
            PhpType::AssocArray {
                key: Box::new(PhpType::Mixed),
                value: Box::new(PhpType::Mixed),
            },
            Op::MixedToHash.default_effects(),
            Some(span),
        )
        .value;
    if ctx.value_is_owning_temporary(source) {
        crate::ir_lower::ownership::release_if_owned(ctx, source, Some(span));
    }
    converted
}

/// Promotes the OpenSSL encrypt tag target to string-capable storage before lowering its load.
fn prepare_openssl_encrypt_tag_local(ctx: &mut LoweringContext<'_, '_>, args: &[Expr]) {
    let expanded = crate::types::call_args::expand_static_assoc_spread_args(args);
    let tag = expanded
        .iter()
        .find_map(|arg| match &arg.kind {
            ExprKind::NamedArg { name, value } if php_symbol_key(name) == "tag" => {
                Some(value.as_ref())
            }
            _ => None,
        })
        .or_else(|| {
            expanded
                .get(5)
                .filter(|arg| !matches!(arg.kind, ExprKind::NamedArg { .. }))
        });
    let Some(Expr {
        kind: ExprKind::Variable(name),
        ..
    }) = tag
    else {
        return;
    };
    ctx.set_local_type(name, PhpType::Str);
}

/// Lowers plain positional builtin operands without materializing omitted defaults or packing tails.
///
/// Runtime helpers consume the caller-provided arity, while the registry signature still supplies
/// by-reference handling and scalar storage coercions for every visible regular parameter.
pub(super) fn lower_positional_builtin_args_with_signature(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    let Some(sig) = sig else {
        return lower_args(ctx, args);
    };
    let regular_param_count = crate::types::call_args::regular_param_count(sig);
    args.iter()
        .enumerate()
        .map(|(index, arg)| {
            if index < regular_param_count {
                lower_arg_with_signature(ctx, sig, index, arg)
            } else {
                lower_expr(ctx, arg).value
            }
        })
        .collect()
}

/// Preserves a boxed nullable name while reusing shared named and spread argument planning.
fn lower_getenv_args(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    let mut sig = sig.cloned();
    if let Some(sig) = sig.as_mut() {
        crate::ir::RuntimeFnId::Getenv.refine_first_class_callable_sig(sig);
    }
    if !crate::types::call_args::has_named_args(args) && !args.iter().any(is_spread_arg) {
        lower_positional_builtin_args_with_signature(ctx, sig.as_ref(), args)
    } else {
        lower_args_with_signature(ctx, sig.as_ref(), args)
    }
}

/// Promotes a packed local before `krsort()` so descending iteration can preserve integer keys.
///
/// Packed storage has no independent iteration-order metadata: reversing its slots would also
/// change `$array[0]`. Converting the by-reference local to hash storage keeps each key/value pair
/// intact while allowing the runtime helper to reorder only the insertion-order links.
fn lower_reverse_key_sort_args(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    let Some(sig) = sig else {
        return lower_args(ctx, args);
    };
    if args.len() == 1 && !args.iter().any(is_spread_arg) {
        let arg = match &args[0].kind {
            ExprKind::NamedArg { value, .. } => value.as_ref(),
            _ => &args[0],
        };
        if let Some(value) = lower_indexed_array_ref_arg_to_hash(ctx, sig, 0, arg) {
            return vec![value];
        }
    }
    lower_args_with_signature(ctx, Some(sig), args)
}

/// Converts one packed by-reference local argument into key-preserving associative storage.
fn lower_indexed_array_ref_arg_to_hash(
    ctx: &mut LoweringContext<'_, '_>,
    sig: &FunctionSig,
    index: usize,
    arg: &Expr,
) -> Option<crate::ir::ValueId> {
    if !sig.ref_params.get(index).copied().unwrap_or(false) {
        return None;
    }
    let ExprKind::Variable(name) = &arg.kind else {
        return None;
    };
    let PhpType::Array(elem_ty) = ctx.local_type(name).codegen_repr() else {
        return None;
    };
    let assoc_ty = PhpType::AssocArray {
        key: Box::new(PhpType::Int),
        value: elem_ty,
    };
    let array = ctx.load_local(name, Some(arg.span));
    ctx.prepare_mutated_local_owner(name, array, assoc_ty.clone(), Some(arg.span));
    let hash = ctx.emit_value(
        Op::ArrayToHash,
        vec![array.value],
        None,
        assoc_ty.clone(),
        Op::ArrayToHash.default_effects(),
        Some(arg.span),
    );
    ctx.store_prepared_mutated_local(name, hash, assoc_ty, Some(arg.span));
    Some(ctx.load_local(name, Some(arg.span)).value)
}

/// Lowers `count()` arguments, dropping a statically-default mode argument.
///
/// The EIR backend implements only `COUNT_NORMAL`; a literal `0` mode (named
/// or positional) is semantically a no-op and would otherwise trip the unary
/// count contract in codegen.
pub(super) fn lower_count_args(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    let pruned: Vec<Expr> = args
        .iter()
        .enumerate()
        .filter(|(index, arg)| !count_arg_is_static_default_mode(*index, arg))
        .map(|(_, arg)| arg.clone())
        .collect();
    let mut operands = lower_args_with_signature(ctx, sig, &pruned);
    // Named and spread plans re-materialize the optional `mode` default even
    // after the AST prune; a trailing constant-zero mode stays a no-op for
    // the unary count contract, so drop the operand (DCE reclaims the const).
    if operands.len() == 2 {
        let trailing_zero_mode = ctx
            .builder
            .value_defining_instruction(operands[1])
            .is_some_and(|inst| {
                inst.op == Op::ConstI64
                    && matches!(inst.immediate, Some(crate::ir::Immediate::I64(0)))
            });
        if trailing_zero_mode {
            operands.pop();
        }
    }
    operands
}

/// Returns true when a `count()` argument is a statically-zero mode.
pub(super) fn count_arg_is_static_default_mode(index: usize, arg: &Expr) -> bool {
    match &arg.kind {
        ExprKind::NamedArg { name, value } => {
            name == "mode" && matches!(value.kind, ExprKind::IntLiteral(0))
        }
        ExprKind::IntLiteral(0) => index == 1,
        _ => false,
    }
}

/// Lowers eval's code operand and coerces it through PHP string-conversion rules.
pub(super) fn lower_eval_args(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    let operands = lower_args_with_signature(ctx, sig, args);
    let Some(code) = operands.first().copied() else {
        return operands;
    };
    let code_value = LoweredValue {
        value: code,
        ir_type: ctx.builder.value_type(code),
    };
    let span = args.first().map(|arg| arg.span);
    vec![coerce_to_string_at_span(ctx, code_value, span).value]
}

/// Lowers `usort`/`uasort` arguments, typing an unannotated comparator closure
/// against the array's object element type.
///
/// `usort`/`uasort` compare values, so a comparator over an array of objects must
/// see each element as the object handle — for `<=>` instant comparison and for
/// property/method access — not the raw pointer-sized integer the runtime stores
/// in each slot. The array operand is lowered exactly as the default positional
/// path would (positional builtin calls reach here with no signature); only an
/// unannotated closure comparator over an object-element array is specialized,
/// matching the element-type hint the checker applied to the comparator body.
pub(super) fn lower_user_value_sort_args(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    if args.len() != 2 || !matches!(&args[1].kind, ExprKind::Closure { .. }) {
        return lower_args_with_signature(ctx, sig, args);
    }
    // The mutating sort keeps its by-reference local storeback in the EIR backend,
    // so the array operand only has to resolve to the array's value here.
    let array = match sig {
        Some(sig) => lower_arg_with_signature(ctx, sig, 0, &args[0]),
        None => lower_expr(ctx, &args[0]).value,
    };
    let elem_ty = match ctx.builder.value_php_type(array).codegen_repr() {
        PhpType::Array(elem) => elem.codegen_repr(),
        _ => PhpType::Int,
    };
    // Only an object-element array needs the comparator parameters re-typed; scalar
    // comparators already lower correctly through the default path.
    let callback = if matches!(elem_ty, PhpType::Object(_)) {
        lower_value_sort_comparator_closure(ctx, &args[1], elem_ty)
    } else {
        match sig {
            Some(sig) => lower_arg_with_signature(ctx, sig, 1, &args[1]),
            None => lower_expr(ctx, &args[1]).value,
        }
    };
    vec![array, callback]
}

/// Lowers a value-sort comparator closure with both parameters typed as the array element.
///
/// Falls back to the plain closure lowering for any non-closure callback operand,
/// though callers only reach this path with a closure comparator.
pub(super) fn lower_value_sort_comparator_closure(
    ctx: &mut LoweringContext<'_, '_>,
    callback: &Expr,
    elem_ty: PhpType,
) -> crate::ir::ValueId {
    let ExprKind::Closure {
        params,
        variadic,
        variadic_by_ref,
        return_type,
        body,
        captures,
        capture_refs,
        is_static,
        ..
    } = &callback.kind
    else {
        return lower_expr(ctx, callback).value;
    };
    lower_closure_with_context(
        ctx,
        params,
        variadic.as_deref(),
        *variadic_by_ref,
        return_type.as_ref(),
        body,
        captures,
        capture_refs,
        callback,
        &[elem_ty.clone(), elem_ty],
        None,
        *is_static,
    )
    .value
}
