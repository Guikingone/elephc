//! Purpose:
//! Lowers static and runtime PHP type predicates, including iterable checks.
//!
//! Called from:
//! - `super` EIR type-predicate and runtime-function dispatch.
//!
//! Key details:
//! - Uses boxed Mixed tags and interface metadata consistently across supported targets.

use super::*;
use crate::codegen_support::callable_descriptor;

/// Lowers the reusable EIR PHP type predicate through target-aware value inspection.
pub(crate) fn lower_type_predicate(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let Some(Immediate::TypePredicate(predicate)) = inst.immediate else {
        return Err(CodegenIrError::unsupported(
            "type_predicate requires a typed predicate immediate",
        ));
    };
    match predicate {
        PhpTypePredicate::Array => lower_is_array(ctx, inst),
        PhpTypePredicate::Bool => {
            lower_static_type_predicate(ctx, inst, "type_predicate", PhpType::Bool)
        }
        PhpTypePredicate::Float => {
            lower_static_type_predicate(ctx, inst, "type_predicate", PhpType::Float)
        }
        PhpTypePredicate::Int => {
            lower_static_type_predicate(ctx, inst, "type_predicate", PhpType::Int)
        }
        PhpTypePredicate::Iterable => lower_is_iterable(ctx, inst),
        PhpTypePredicate::Object => lower_is_object(ctx, inst),
        PhpTypePredicate::Resource => types::lower_is_resource(ctx, inst),
        PhpTypePredicate::Scalar => lower_is_scalar(ctx, inst),
        PhpTypePredicate::String => {
            lower_static_type_predicate(ctx, inst, "type_predicate", PhpType::Str)
        }
    }
}

/// Lowers a static PHP type predicate for concrete non-Mixed values.
pub(crate) fn lower_static_type_predicate(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
    expected: PhpType,
) -> Result<()> {
    ensure_arg_count(inst, name, 1)?;
    let value = expect_operand(inst, 0)?;
    let ty = ctx.value_php_type(value)?;
    if ty == PhpType::TaggedScalar {
        if expected == PhpType::Int {
            emit_tagged_scalar_int_predicate(ctx, value)?;
        } else {
            emit_static_bool(ctx, false);
        }
        return store_if_result(ctx, inst);
    }
    if matches!(ty, PhpType::Mixed | PhpType::Union(_)) {
        if let Some(tag) = mixed_type_predicate_tag(&expected) {
            predicates::emit_mixed_tag_eq(ctx, value, tag)?;
        } else {
            emit_static_bool(ctx, false);
        }
        return store_if_result(ctx, inst);
    }
    emit_static_bool(ctx, ty == expected);
    store_if_result(ctx, inst)
}

/// Emits `is_int()` for a tagged scalar by checking that its tag is not null.
pub(in crate::codegen::lower_inst) fn emit_tagged_scalar_int_predicate(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
) -> Result<()> {
    ctx.load_value_to_result(value)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            let cmp_inst = format!(
                "cmp x1, #{}",
                crate::codegen::sentinels::TAGGED_SCALAR_TAG_NULL
            );
            ctx.emitter.instruction(&cmp_inst);                                 // does the tagged scalar carry the runtime null tag?
            ctx.emitter.instruction("cset x0, ne");                             // materialize true when the tagged scalar holds an integer
        }
        Arch::X86_64 => {
            let cmp_inst = format!(
                "cmp rdx, {}",
                crate::codegen::sentinels::TAGGED_SCALAR_TAG_NULL
            );
            ctx.emitter.instruction(&cmp_inst);                                 // does the tagged scalar carry the runtime null tag?
            ctx.emitter.instruction("setne al");                                // materialize true when the tagged scalar holds an integer
            ctx.emitter.instruction("movzx rax, al");                           // widen the boolean byte into the integer result register
        }
    }
    Ok(())
}

/// Lowers `is_iterable()` for concrete values and boxed Mixed payloads.
pub(crate) fn lower_is_iterable(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count(inst, "is_iterable", 1)?;
    let value = expect_operand(inst, 0)?;
    let ty = ctx.value_php_type(value)?;
    let result = match ty {
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Iterable => true,
        PhpType::Object(name) => object_type_implements_iterable(ctx, &name),
        PhpType::Int
        | PhpType::Float
        | PhpType::Str
        | PhpType::Bool
        | PhpType::False
        | PhpType::Void
        | PhpType::Never
        | PhpType::Callable
        | PhpType::Pointer(_)
        | PhpType::Buffer(_)
        | PhpType::Packed(_)
        | PhpType::Resource(_)
        | PhpType::TaggedScalar => false,
        PhpType::Mixed | PhpType::Union(_) => {
            emit_mixed_is_iterable(ctx, value)?;
            return store_if_result(ctx, inst);
        }
    };
    emit_static_bool(ctx, result);
    store_if_result(ctx, inst)
}

/// Emits runtime `is_iterable()` checks for a boxed Mixed or Union value.
pub(in crate::codegen::lower_inst) fn emit_mixed_is_iterable(ctx: &mut FunctionContext<'_>, value: ValueId) -> Result<()> {
    let true_case = ctx.next_label("is_iterable_mixed_true");
    let object_case = ctx.next_label("is_iterable_mixed_object");
    let done = ctx.next_label("is_iterable_mixed_done");
    let ty = ctx.load_value_to_result(value)?;
    if !matches!(ty, PhpType::Mixed | PhpType::Union(_)) {
        return Err(CodegenIrError::unsupported(format!(
            "is_iterable Mixed check for PHP type {:?}",
            ty
        )));
    }
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #4");                              // check for a boxed indexed-array payload
            ctx.emitter.instruction(&format!("b.eq {}", true_case));            // indexed arrays satisfy is_iterable
            ctx.emitter.instruction("cmp x0, #5");                              // check for a boxed associative-array payload
            ctx.emitter.instruction(&format!("b.eq {}", true_case));            // associative arrays satisfy is_iterable
            ctx.emitter.instruction("cmp x0, #6");                              // check for a boxed object payload
            ctx.emitter.instruction(&format!("b.eq {}", object_case));          // objects need a Traversable interface check
            ctx.emitter.instruction("mov x0, #0");                              // all other Mixed payloads are not iterable
            ctx.emitter.instruction(&format!("b {}", done));                    // skip the truthy result path
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 4");                              // check for a boxed indexed-array payload
            ctx.emitter.instruction(&format!("je {}", true_case));              // indexed arrays satisfy is_iterable
            ctx.emitter.instruction("cmp rax, 5");                              // check for a boxed associative-array payload
            ctx.emitter.instruction(&format!("je {}", true_case));              // associative arrays satisfy is_iterable
            ctx.emitter.instruction("cmp rax, 6");                              // check for a boxed object payload
            ctx.emitter.instruction(&format!("je {}", object_case));            // objects need a Traversable interface check
            ctx.emitter.instruction("mov rax, 0");                              // all other Mixed payloads are not iterable
            ctx.emitter.instruction(&format!("jmp {}", done));                  // skip the truthy result path
        }
    }
    ctx.emitter.label(&object_case);
    emit_runtime_object_iterable_check(ctx, &true_case, &done);
    ctx.emitter.label(&true_case);
    abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 1);
    ctx.emitter.label(&done);
    Ok(())
}

/// Emits the object half of runtime `is_iterable()` by checking Traversable interfaces.
pub(in crate::codegen::lower_inst) fn emit_runtime_object_iterable_check(
    ctx: &mut FunctionContext<'_>,
    true_case: &str,
    done: &str,
) {
    let object_true = ctx.next_label("is_iterable_object_true");
    let interface_ids = traversable_interface_ids(ctx);
    if interface_ids.is_empty() {
        abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
        abi::emit_jump(ctx.emitter, done);
        return;
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("str x1, [sp, #-16]!");                     // preserve the unboxed object pointer across Traversable checks
            for interface_id in interface_ids {
                emit_saved_object_interface_check(ctx, interface_id, &object_true);
            }
            ctx.emitter.instruction("add sp, sp, #16");                         // discard the saved object pointer after failed checks
            ctx.emitter.instruction("mov x0, #0");                              // non-Traversable objects are not iterable
            ctx.emitter.instruction(&format!("b {}", done));                    // skip the truthy result path
            ctx.emitter.label(&object_true);
            ctx.emitter.instruction("add sp, sp, #16");                         // discard the saved object pointer before returning true
            ctx.emitter.instruction(&format!("b {}", true_case));               // continue through the shared truthy result path
        }
        Arch::X86_64 => {
            abi::emit_push_reg(ctx.emitter, "rdi");
            for interface_id in interface_ids {
                emit_saved_object_interface_check(ctx, interface_id, &object_true);
            }
            abi::emit_pop_reg(ctx.emitter, "r10");
            ctx.emitter.instruction("xor eax, eax");                            // non-Traversable objects are not iterable
            ctx.emitter.instruction(&format!("jmp {}", done));                  // skip the truthy result path
            ctx.emitter.label(&object_true);
            abi::emit_pop_reg(ctx.emitter, "r10");
            ctx.emitter.instruction(&format!("jmp {}", true_case));             // continue through the shared truthy result path
        }
    }
}

/// Emits one interface matcher call for a saved object pointer.
pub(in crate::codegen::lower_inst) fn emit_saved_object_interface_check(
    ctx: &mut FunctionContext<'_>,
    interface_id: u64,
    true_case: &str,
) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("ldr x0, [sp]");                            // reload the object pointer as matcher argument 1
            abi::emit_load_int_immediate(ctx.emitter, "x1", interface_id as i64);
            abi::emit_load_int_immediate(ctx.emitter, "x2", 1);
            abi::emit_call_label(ctx.emitter, "__rt_exception_matches"); // check whether the object implements the Traversable interface
            ctx.emitter.instruction("cmp x0, #0");                              // test whether the runtime matcher succeeded
            ctx.emitter.instruction(&format!("b.ne {}", true_case));            // a matching interface makes the object iterable
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp]");                // reload the object pointer as matcher argument 1
            abi::emit_load_int_immediate(ctx.emitter, "rsi", interface_id as i64);
            abi::emit_load_int_immediate(ctx.emitter, "rdx", 1);
            abi::emit_call_label(ctx.emitter, "__rt_exception_matches"); // check whether the object implements the Traversable interface
            ctx.emitter.instruction("test rax, rax");                           // test whether the runtime matcher succeeded
            ctx.emitter.instruction(&format!("jne {}", true_case));             // a matching interface makes the object iterable
        }
    }
}

/// Returns runtime interface IDs for the interfaces that make an object iterable.
pub(in crate::codegen::lower_inst) fn traversable_interface_ids(ctx: &FunctionContext<'_>) -> Vec<u64> {
    ["Iterator", "IteratorAggregate"]
        .into_iter()
        .filter_map(|name| {
            ctx.module
                .interface_infos
                .get(name)
                .map(|info| info.interface_id)
        })
        .collect()
}

/// Returns whether a statically known class or interface satisfies `is_iterable()`.
pub(in crate::codegen::lower_inst) fn object_type_implements_iterable(ctx: &FunctionContext<'_>, type_name: &str) -> bool {
    let normalized = normalized_type_name(type_name);
    if let Some(class_info) = ctx.module.class_infos.get(normalized) {
        return class_info.interfaces.iter().any(|interface_name| {
            is_traversable_interface_name(interface_name)
                || interface_extends_traversable(ctx, interface_name)
        });
    }
    if ctx.module.interface_infos.contains_key(normalized) {
        return is_traversable_interface_name(normalized)
            || interface_extends_traversable(ctx, normalized);
    }
    false
}

/// Returns whether an interface name is one of PHP's Traversable contracts.
pub(in crate::codegen::lower_inst) fn is_traversable_interface_name(interface_name: &str) -> bool {
    let key = php_symbol_key(normalized_type_name(interface_name));
    key == php_symbol_key("Iterator") || key == php_symbol_key("IteratorAggregate")
}

/// Returns whether an interface extends Iterator or IteratorAggregate.
pub(in crate::codegen::lower_inst) fn interface_extends_traversable(ctx: &FunctionContext<'_>, interface_name: &str) -> bool {
    let mut stack = vec![normalized_type_name(interface_name).to_string()];
    while let Some(current) = stack.pop() {
        if is_traversable_interface_name(&current) {
            return true;
        }
        if let Some(interface_info) = ctx.module.interface_infos.get(&current) {
            stack.extend(
                interface_info
                    .parents
                    .iter()
                    .map(|parent| normalized_type_name(parent).to_string()),
            );
        }
    }
    false
}

/// Normalizes a PHP class or interface name for metadata lookups.
pub(in crate::codegen::lower_inst) fn normalized_type_name(type_name: &str) -> &str {
    type_name.trim_start_matches('\\')
}

/// Lowers `is_array()`: true for statically-known arrays/hashes, or a boxed Mixed/Union value
/// whose runtime tag is an indexed (4) or associative (5) array. An `iterable`-typed value holds
/// either shape or a Traversable object, so it is decided at RUN TIME from its heap kind.
pub(crate) fn lower_is_array(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count(inst, "is_array", 1)?;
    let value = expect_operand(inst, 0)?;
    match ctx.value_php_type(value)? {
        PhpType::Array(_) | PhpType::AssocArray { .. } => emit_static_bool(ctx, true),
        PhpType::Callable => emit_callable_array_predicate(ctx, value)?,
        PhpType::Mixed | PhpType::Union(_) => {
            emit_mixed_array_predicate(ctx, value)?;
        }
        PhpType::Iterable => emit_iterable_heap_kind_predicate(ctx, value, IterableKind::Array)?,
        _ => emit_static_bool(ctx, false),
    }
    store_if_result(ctx, inst)
}

/// Which side of `iterable`'s `array|Traversable` an `is_*` predicate is asking about.
#[derive(Clone, Copy)]
enum IterableKind {
    /// PHP `array`: heap kind 2 (indexed) or 3 (hash).
    Array,
    /// A `Traversable` instance: heap kind 4.
    Object,
}

/// Answers an `is_array`/`is_object` predicate for a value whose static type is `iterable`.
///
/// `iterable` is `array|Traversable`, and unlike `Mixed` it is NOT a boxed cell carrying a tag:
/// `PhpType::Iterable` is a raw heap pointer whose concrete shape is read from the uniform heap
/// header (`crate::types::model` — "type-erased pointer (array|Traversable)"). The predicate
/// therefore has to ask the heap, which is what `__rt_heap_kind` is for; it is the same probe
/// `codegen_support::value_boxing::emit_box_iterable_as_mixed` uses to give an iterable a Mixed
/// tag, and it maps kinds the same way — 2 indexed array, 3 hash, 4 object.
///
/// Answering statically instead was measured wrong on the commonest shape there is: a promoted
/// constructor property `private iterable $items = []` holding a plain array reported
/// `is_array=false` while `get_debug_type` on the same value said `array`, so a `!is_array($x)`
/// guard took the branch PHP never takes. `count()` on the same value has always dispatched on
/// the heap kind (`builtins::count_empty::lower_iterable_count`), which is why the two disagreed.
fn emit_iterable_heap_kind_predicate(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    kind: IterableKind,
) -> Result<()> {
    ctx.load_value_to_result(value)?;
    abi::emit_call_label(ctx.emitter, "__rt_heap_kind");
    match (ctx.emitter.target.arch, kind) {
        // Kinds 2 and 3 are the two array shapes, so one unsigned range test covers both.
        (Arch::AArch64, IterableKind::Array) => {
            ctx.emitter.instruction("sub x0, x0, #2");                          // rebase the indexed/hash heap kinds onto 0 and 1
            ctx.emitter.instruction("cmp x0, #1");                              // is the rebased heap kind still inside the array pair?
            ctx.emitter.instruction("cset x0, ls");                             // materialize true for an indexed array or a hash
        }
        (Arch::AArch64, IterableKind::Object) => {
            ctx.emitter.instruction("cmp x0, #4");                              // heap kind 4 is an object instance
            ctx.emitter.instruction("cset x0, eq");                             // materialize true only for a Traversable instance
        }
        (Arch::X86_64, IterableKind::Array) => {
            ctx.emitter.instruction("sub rax, 2");                              // rebase the indexed/hash heap kinds onto 0 and 1
            ctx.emitter.instruction("cmp rax, 1");                              // is the rebased heap kind still inside the array pair?
            ctx.emitter.instruction("setbe al");                                // materialize true for an indexed array or a hash
            ctx.emitter.instruction("movzx rax, al");                           // widen the boolean byte into the integer result register
        }
        (Arch::X86_64, IterableKind::Object) => {
            ctx.emitter.instruction("cmp rax, 4");                              // heap kind 4 is an object instance
            ctx.emitter.instruction("sete al");                                 // materialize true only for a Traversable instance
            ctx.emitter.instruction("movzx rax, al");                           // widen the boolean byte into the integer result register
        }
    }
    Ok(())
}

/// Checks boxed gradual values for indexed/hash arrays or a callable descriptor explicitly marked
/// as originating from PHP callable-array syntax.
fn emit_mixed_array_predicate(ctx: &mut FunctionContext<'_>, value: ValueId) -> Result<()> {
    let true_label = ctx.next_label("is_array_mixed_true");
    let callable_label = ctx.next_label("is_array_mixed_callable");
    let done_label = ctx.next_label("is_array_mixed_done");
    ctx.load_value_to_result(value)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #4");                              // indexed array runtime tag
            ctx.emitter.instruction(&format!("b.eq {}", true_label));
            ctx.emitter.instruction("cmp x0, #5");                              // associative array runtime tag
            ctx.emitter.instruction(&format!("b.eq {}", true_label));
            ctx.emitter.instruction("cmp x0, #10");                             // callable descriptor runtime tag
            ctx.emitter.instruction(&format!("b.eq {}", callable_label));
            ctx.emitter.instruction("mov x0, #0");                              // all other gradual values are not arrays
            ctx.emitter.instruction(&format!("b {}", done_label));
            ctx.emitter.label(&callable_label);
            ctx.emitter.instruction("ldr x9, [x1]");                            // load the callable descriptor source-shape kind
            ctx.emitter.instruction(&format!(
                "cmp x9, #{}",
                callable_descriptor::CALLABLE_DESC_KIND_ARRAY
            ));
            ctx.emitter.instruction("cset x0, eq");
            ctx.emitter.instruction(&format!("b {}", done_label));
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 4");                              // indexed array runtime tag
            ctx.emitter.instruction(&format!("je {}", true_label));
            ctx.emitter.instruction("cmp rax, 5");                              // associative array runtime tag
            ctx.emitter.instruction(&format!("je {}", true_label));
            ctx.emitter.instruction("cmp rax, 10");                             // callable descriptor runtime tag
            ctx.emitter.instruction(&format!("je {}", callable_label));
            ctx.emitter.instruction("xor eax, eax");                            // all other gradual values are not arrays
            ctx.emitter.instruction(&format!("jmp {}", done_label));
            ctx.emitter.label(&callable_label);
            ctx.emitter.instruction("cmp QWORD PTR [rdi], 5");                  // compare the descriptor source-shape kind
            ctx.emitter.instruction("sete al");
            ctx.emitter.instruction("movzx rax, al");
            ctx.emitter.instruction(&format!("jmp {}", done_label));
        }
    }
    ctx.emitter.label(&true_label);
    abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 1);
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Checks whether a compact callable descriptor originated from PHP's two-element callable-array
/// syntax rather than a closure, function string, or first-class callable.
fn emit_callable_array_predicate(ctx: &mut FunctionContext<'_>, value: ValueId) -> Result<()> {
    ctx.load_value_to_result(value)?;
    let descriptor_reg = abi::int_result_reg(ctx.emitter);
    let kind_reg = abi::secondary_scratch_reg(ctx.emitter);
    abi::emit_load_from_address(ctx.emitter, kind_reg, descriptor_reg, 0);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!(
                "cmp {}, #{}",
                kind_reg,
                callable_descriptor::CALLABLE_DESC_KIND_ARRAY
            ));
            ctx.emitter.instruction("cset x0, eq");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!(
                "cmp {}, {}",
                kind_reg,
                callable_descriptor::CALLABLE_DESC_KIND_ARRAY
            ));
            ctx.emitter.instruction("sete al");
            ctx.emitter.instruction("movzx rax, al");
        }
    }
    Ok(())
}

/// Lowers `is_object()`: true for statically-known objects and closures, or boxed values whose
/// runtime tag is an object (6) or callable descriptor (10). An `iterable` is the other half of
/// the same run-time question `is_array` asks — see [`emit_iterable_heap_kind_predicate`].
pub(crate) fn lower_is_object(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count(inst, "is_object", 1)?;
    let value = expect_operand(inst, 0)?;
    match ctx.value_php_type(value)? {
        PhpType::Object(_) | PhpType::Callable => emit_static_bool(ctx, true),
        PhpType::Mixed | PhpType::Union(_) => {
            predicates::emit_mixed_tag_membership(ctx, value, &[6, 10])?;
        }
        PhpType::Iterable => emit_iterable_heap_kind_predicate(ctx, value, IterableKind::Object)?,
        _ => emit_static_bool(ctx, false),
    }
    store_if_result(ctx, inst)
}

/// Lowers `is_scalar()`: true for int/float/string/bool, a non-null tagged scalar, or a boxed
/// Mixed/Union value whose runtime tag is int (0), string (1), float (2), or bool (3). Null,
/// arrays, objects, and resources are not scalars, matching PHP.
pub(crate) fn lower_is_scalar(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count(inst, "is_scalar", 1)?;
    let value = expect_operand(inst, 0)?;
    match ctx.value_php_type(value)? {
        PhpType::Int | PhpType::Float | PhpType::Str | PhpType::Bool | PhpType::False => {
            emit_static_bool(ctx, true)
        }
        PhpType::TaggedScalar => emit_tagged_scalar_int_predicate(ctx, value)?,
        PhpType::Mixed | PhpType::Union(_) => {
            predicates::emit_mixed_tag_membership(ctx, value, &[0, 1, 2, 3])?;
        }
        _ => emit_static_bool(ctx, false),
    }
    store_if_result(ctx, inst)
}

/// Returns the runtime Mixed tag used by a supported type predicate.
pub(in crate::codegen::lower_inst) fn mixed_type_predicate_tag(expected: &PhpType) -> Option<u8> {
    match expected {
        PhpType::Int => Some(0),
        PhpType::Str => Some(1),
        PhpType::Float => Some(2),
        PhpType::Bool | PhpType::False => Some(3),
        _ => None,
    }
}
