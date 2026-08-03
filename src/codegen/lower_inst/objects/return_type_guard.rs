//! Purpose:
//! Emits the target-aware assembly for the checked-downcast guards' throw ops — the runtime half
//! of the chain `crate::ir_lower::checked_downcast` emits, entered only when every declared arm
//! mismatched. `Op::ThrowCheckedReturnTypeError` serves the RETURN position;
//! `Op::ThrowCheckedTypeError` serves the argument position.
//!
//! Called from:
//! - `crate::codegen::lower_inst::objects::lower_throw_checked_return_type_error()`.
//! - `crate::codegen::lower_inst::objects::lower_throw_checked_type_error()`.
//!
//! Key details:
//! - Builds `"<prefix><actual runtime type name><suffix>"` at runtime (the prefix — callee/owner
//!   name + declared type — is a compile-time string baked by `ir_lower`; the type name is looked
//!   up dynamically, per the jury addendum requiring the ACTUAL runtime class, never a static
//!   approximation) via two `__rt_concat` calls plus `__rt_str_persist`, then mirrors
//!   `objects::reflection::emit_reflection_class_argument_type_error_throw`'s
//!   allocation/publish/unwind tail exactly (stamps `_spl_type_error_class_id`). Never returns.
//! - Only the RETURN op releases the mismatched value (it is never returned to the caller, so
//!   nothing else owns it). The argument op must NOT: the caller's local still owns the value,
//!   and releasing it there is a double free. That split is by OP, and stays by op — but WHICH
//!   release helper the return op emits is a REPRESENTATION choice made from the operand's own
//!   type (`__rt_decref_object` for a raw object, `__rt_decref_mixed` for a box), which is a
//!   different axis from the ownership policy and must not be confused with it.
//! - A BOXED source needs its own tag→name table (int/string/float/true/false/null/array/
//!   <runtime class>/Closure), and BOTH positions now use it — a box has no object header for
//!   `get_class` to read. It deliberately does NOT reuse
//!   `builtins::arrays::union_type_guard::emit_mixed_wrong_tag_type_error_dispatch`, whose
//!   generic bucket maps tags 4/5/6 alike to the literal word `object` and therefore already
//!   mis-names an ARRAY payload for the array builtins that use it.

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::ir::ValueId;
use crate::types::PhpType;

use super::super::super::context::FunctionContext;
use crate::codegen::Result;

/// Returns a register pair guaranteed NOT to collide with `abi::int_result_reg` — used to
/// reload the parked message across the object allocation, once `int_result_reg` holds the
/// freshly allocated exception object pointer. Mirrors the scratch pair
/// `objects::reflection::emit_reflection_class_argument_type_error_throw` reloads into.
fn message_scratch_regs(ctx: &FunctionContext<'_>) -> (&'static str, &'static str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => ("x9", "x10"),
        Arch::X86_64 => ("r10", "r11"),
    }
}

/// Returns the AArch64/x86_64 `__rt_concat` left-operand (also its output) register pair.
fn concat_lhs_regs(ctx: &FunctionContext<'_>) -> (&'static str, &'static str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => ("x1", "x2"),
        Arch::X86_64 => ("rax", "rdx"),
    }
}

/// Returns the AArch64/x86_64 `__rt_concat` right-operand register pair.
fn concat_rhs_regs(ctx: &FunctionContext<'_>) -> (&'static str, &'static str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => ("x3", "x4"),
        Arch::X86_64 => ("rdi", "rsi"),
    }
}

/// Loads a compile-time interned string's (ptr, len) into the given register pair.
fn load_static_string_into(ctx: &mut FunctionContext<'_>, label: &str, len: usize, ptr_reg: &str, len_reg: &str) {
    abi::emit_symbol_address(ctx.emitter, ptr_reg, label);
    abi::emit_load_int_immediate(ctx.emitter, len_reg, len as i64);
}

/// Builds `"<prefix><actual runtime type name> returned"` and throws it as a catchable
/// `\TypeError`, RELEASING the mismatched value.
///
/// On entry, the mismatched value must already be loaded in the target's integer result register
/// (the caller — `lower_throw_checked_return_type_error` — does this via
/// `ctx.load_value_to_result`). `prefix_label`/`prefix_len` name the compile-time message prefix
/// (function name + declared type, already formatted by `ir_lower`). Never returns.
///
/// `value`/`value_ty` are the guarded operand and its static type, and BOTH of the two things that
/// vary with it are representation choices, not policy ones — the release itself is unconditional
/// at this position, because a return value the caller never receives has no other owner:
/// - THE NAME. A raw object source resolves its class through `get_class`; a BOXED source cannot,
///   because `get_class` would read a `Mixed` cell's first word as an object header. It unboxes and
///   dispatches on the runtime tag instead, through the same table the argument position uses — so
///   a boxed `null` reaching a declared `D` return is named `null`, exactly as PHP names it.
/// - THE RELEASE HELPER. `emit_decref_if_refcounted` is handed the value's own representation, so a
///   raw object goes to `__rt_decref_object` (byte-for-byte what this emitted before a boxed source
///   could reach it) and a box goes to `__rt_decref_mixed`. Releasing a `Mixed` cell as though it
///   were an object would decrement a refcount word that is the cell's TAG.
pub(super) fn emit_throw_checked_return_type_error(
    ctx: &mut FunctionContext<'_>,
    prefix_label: &str,
    prefix_len: usize,
    value: ValueId,
    value_ty: &PhpType,
    suffix: Option<ValueId>,
) -> Result<()> {
    let source_is_raw_object = matches!(
        value_ty.codegen_repr(),
        PhpType::Object(_) | PhpType::Packed(_)
    );
    let object_reg = abi::int_result_reg(ctx.emitter);
    // The RETURN position has no suffix operand and keeps its baked `" returned"`, which is what
    // keeps its emitted assembly byte-for-byte what it was. A position that supplies one (the
    // property store, whose PHP wording ends in the property name and the declared type) has its
    // suffix materialized into the same static-string slot instead.
    let baked_suffix = if suffix.is_none() {
        Some(ctx.data.add_string(b" returned"))
    } else {
        None
    };

    // -- park the mismatched value across the message-building/allocation calls, so it can be
    //    released once the exception object no longer needs it --
    abi::emit_push_reg(ctx.emitter, object_reg);

    // -- resolve the mismatched value's ACTUAL runtime type name (never a static approximation),
    //    leaving it in the concat right-operand pair before the compile-time prefix overwrites
    //    the concat lhs/output slot --
    if source_is_raw_object {
        emit_runtime_class_name_into_concat_rhs(ctx);
    } else {
        emit_boxed_type_name_into_concat_rhs(ctx, value)?;
    }
    let (lhs_ptr, lhs_len) = concat_lhs_regs(ctx);

    // -- prefix ("F(): Return value must be of type D, ") + actual type name --
    load_static_string_into(ctx, prefix_label, prefix_len, lhs_ptr, lhs_len);
    abi::emit_call_label(ctx.emitter, "__rt_concat");

    // -- append the position's suffix --
    let (rhs_ptr, rhs_len) = concat_rhs_regs(ctx);
    match (&baked_suffix, suffix) {
        (Some((label, len)), _) => load_static_string_into(ctx, label, *len, rhs_ptr, rhs_len),
        (None, Some(suffix)) => ctx.load_string_value_to_regs(suffix, rhs_ptr, rhs_len)?,
        (None, None) => unreachable!("one of the two suffix sources is always present"),
    }
    abi::emit_call_label(ctx.emitter, "__rt_concat");

    // -- own a permanent heap copy of the assembled message --
    emit_persist_from_concat_result(ctx);
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    abi::emit_push_reg_pair(ctx.emitter, ptr_reg, len_reg);

    // -- allocate and stamp the TypeError object (mirrors
    //    `objects::reflection::emit_reflection_class_argument_type_error_throw`) --
    emit_alloc_type_error_header(ctx);
    // Reload the parked message into DEDICATED scratch registers, never `string_result_regs`:
    // on x86_64 that pair IS `(rax, rdx)`, and `rax` already holds the freshly allocated
    // exception object pointer at this point (`int_result_reg` == `rax` on x86_64, unlike
    // AArch64 where the object pointer sits in `x0`, distinct from `(x1, x2)`) — reusing it
    // here would silently clobber the object pointer before its fields are stored.
    let (ptr_reg, len_reg) = message_scratch_regs(ctx);
    abi::emit_pop_reg_pair(ctx.emitter, ptr_reg, len_reg);
    emit_store_message_fields(ctx, ptr_reg, len_reg);

    // -- release the mismatched value: it is never returned to the caller, so this guard is its
    //    only owner. The helper is chosen by REPRESENTATION — a box is released as a `Mixed` cell,
    //    never as an object. Park the freshly built exception object across the release call. --
    let result_reg = abi::int_result_reg(ctx.emitter);
    let scratch = abi::temp_int_reg(ctx.emitter.target);
    abi::emit_pop_reg(ctx.emitter, scratch); // reload the parked mismatched value pointer
    abi::emit_push_reg(ctx.emitter, result_reg); // park the exception object
    move_reg(ctx, result_reg, scratch);
    let release_ty = if source_is_raw_object {
        PhpType::Object(String::new())
    } else {
        PhpType::Mixed
    };
    abi::emit_decref_if_refcounted(ctx.emitter, &release_ty);
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_pop_reg(ctx.emitter, result_reg); // restore the exception object

    abi::emit_store_reg_to_symbol(ctx.emitter, result_reg, "_exc_value", 0); // publish the active exception object
    abi::emit_jump(ctx.emitter, "__rt_throw_current"); // enter the standard exception unwinder
    Ok(())
}

/// The boxed-`Mixed` runtime tags this guard names, paired with PHP's own spelling in a
/// `TypeError` message (php-8.5.6 verified: `int`/`string`/`float`/`true`/`false`/`null`/`array`/
/// `<class>`/`Closure`). Tag 3 (bool) is split by its payload, and tag 6 resolves the ACTUAL
/// runtime class, so neither appears here.
const BOXED_TAG_NAMES: &[(i64, &str)] = &[
    (0, "int"),
    (1, "string"),
    (2, "float"),
    (4, "array"),
    (5, "array"),
    (8, "null"),
    (10, "Closure"),
];

/// Builds `"<prefix><actual runtime type name><suffix>"` and throws it as a catchable
/// `\TypeError`, WITHOUT releasing the mismatched value.
///
/// `value`/`value_ty` are the guarded operand and its static type: a raw object source resolves
/// its name through `get_class`, a boxed source unboxes and dispatches on the runtime tag.
/// `suffix` is a string operand holding the position's fixed message tail (`" given"`). Never
/// returns.
pub(super) fn emit_throw_checked_type_error(
    ctx: &mut FunctionContext<'_>,
    prefix_label: &str,
    prefix_len: usize,
    value: ValueId,
    value_ty: &PhpType,
    suffix: ValueId,
) -> Result<()> {
    if matches!(value_ty.codegen_repr(), PhpType::Object(_) | PhpType::Packed(_)) {
        ctx.load_value_to_result(value)?;
        emit_runtime_class_name_into_concat_rhs(ctx);
        return emit_message_tail(ctx, prefix_label, prefix_len, suffix);
    }
    emit_boxed_type_name_into_concat_rhs(ctx, value)?;
    emit_message_tail(ctx, prefix_label, prefix_len, suffix)
}

/// Resolves the ACTUAL runtime class of the object currently in the integer result register and
/// leaves its `(ptr, len)` in the `__rt_concat` right-operand pair.
fn emit_runtime_class_name_into_concat_rhs(ctx: &mut FunctionContext<'_>) {
    super::super::builtins::types::emit_dynamic_object_class_name(ctx, "get_class");
    let (lhs_ptr, lhs_len) = concat_lhs_regs(ctx);
    let (rhs_ptr, rhs_len) = concat_rhs_regs(ctx);
    move_reg_pair(ctx, rhs_ptr, rhs_len, lhs_ptr, lhs_len);
}

/// Unboxes `value` and leaves PHP's own name for its runtime payload in the `__rt_concat`
/// right-operand pair: a static spelling for every scalar/array/null/Closure tag, and the ACTUAL
/// runtime class for an object payload.
fn emit_boxed_type_name_into_concat_rhs(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
) -> Result<()> {
    let object_label = ctx.next_label("checked_type_error_object");
    let bool_label = ctx.next_label("checked_type_error_bool");
    let true_label = ctx.next_label("checked_type_error_true");
    let false_label = ctx.next_label("checked_type_error_false");
    let generic_label = ctx.next_label("checked_type_error_generic");
    let done_label = ctx.next_label("checked_type_error_name_done");
    let tag_labels: Vec<(i64, &str, String)> = BOXED_TAG_NAMES
        .iter()
        .map(|(tag, name)| (*tag, *name, ctx.next_label("checked_type_error_tag")))
        .collect();

    // `__rt_mixed_unbox` leaves the runtime tag in the integer result register and the payload's
    // low word in the first argument register — the same convention the array-family guards read.
    let arch = ctx.emitter.target.arch;
    let tag_reg = match arch {
        Arch::AArch64 => "x0",
        Arch::X86_64 => "rax",
    };
    ctx.load_value_to_reg(value, tag_reg)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");                      // tag in x0/rax, payload low word in x1/rdi
    for (tag, _, label) in &tag_labels {
        emit_tag_branch(ctx, tag_reg, *tag, label);
    }
    emit_tag_branch(ctx, tag_reg, 3, &bool_label);
    emit_tag_branch(ctx, tag_reg, 6, &object_label);
    abi::emit_jump(ctx.emitter, &generic_label);                                // any other tag reports the generic `object` spelling

    ctx.emitter.label(&bool_label);
    match arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x1, #0");                              // is the unboxed boolean payload false?
            ctx.emitter.instruction(&format!("b.eq {}", false_label));
            abi::emit_jump(ctx.emitter, &true_label);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rdi, rdi");                           // is the unboxed boolean payload false?
            ctx.emitter.instruction(&format!("je {}", false_label));
            abi::emit_jump(ctx.emitter, &true_label);
        }
    }

    for (_, name, label) in &tag_labels {
        emit_static_type_name_case(ctx, label, name, &done_label);
    }
    emit_static_type_name_case(ctx, &true_label, "true", &done_label);
    emit_static_type_name_case(ctx, &false_label, "false", &done_label);
    emit_static_type_name_case(ctx, &generic_label, "object", &done_label);

    ctx.emitter.label(&object_label);
    match arch {
        Arch::AArch64 => ctx.emitter.instruction("mov x0, x1"),                 // expose the unboxed object pointer to the class-name lookup
        Arch::X86_64 => ctx.emitter.instruction("mov rax, rdi"),                // expose the unboxed object pointer to the class-name lookup
    }
    emit_runtime_class_name_into_concat_rhs(ctx);
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Emits `if tag == <tag> goto <label>` for the active architecture.
fn emit_tag_branch(ctx: &mut FunctionContext<'_>, tag_reg: &str, tag: i64, label: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cmp {}, #{}", tag_reg, tag));     // select the boxed payload's runtime tag
            ctx.emitter.instruction(&format!("b.eq {}", label));
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("cmp {}, {}", tag_reg, tag));      // select the boxed payload's runtime tag
            ctx.emitter.instruction(&format!("je {}", label));
        }
    }
}

/// Emits one tag case: labels it, loads the static PHP spelling into the `__rt_concat`
/// right-operand pair, and jumps to the shared message-building tail.
fn emit_static_type_name_case(
    ctx: &mut FunctionContext<'_>,
    case_label: &str,
    name: &str,
    done_label: &str,
) {
    ctx.emitter.label(case_label);
    let (name_label, name_len) = ctx.data.add_string(name.as_bytes());
    let (rhs_ptr, rhs_len) = concat_rhs_regs(ctx);
    load_static_string_into(ctx, &name_label, name_len, rhs_ptr, rhs_len);
    abi::emit_jump(ctx.emitter, done_label);
}

/// Concatenates `prefix` + the runtime type name already staged in the `__rt_concat` right-operand
/// pair + the `suffix` string operand, then allocates, stamps, publishes and throws the
/// `\TypeError`. Never returns.
fn emit_message_tail(
    ctx: &mut FunctionContext<'_>,
    prefix_label: &str,
    prefix_len: usize,
    suffix: ValueId,
) -> Result<()> {
    let (lhs_ptr, lhs_len) = concat_lhs_regs(ctx);
    load_static_string_into(ctx, prefix_label, prefix_len, lhs_ptr, lhs_len);
    abi::emit_call_label(ctx.emitter, "__rt_concat");

    let (rhs_ptr, rhs_len) = concat_rhs_regs(ctx);
    ctx.load_string_value_to_regs(suffix, rhs_ptr, rhs_len)?;
    abi::emit_call_label(ctx.emitter, "__rt_concat");

    emit_persist_from_concat_result(ctx);
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    abi::emit_push_reg_pair(ctx.emitter, ptr_reg, len_reg);

    emit_alloc_type_error_header(ctx);
    // Reload the parked message into DEDICATED scratch registers, never `string_result_regs`:
    // on x86_64 that pair IS `(rax, rdx)`, and `rax` already holds the freshly allocated
    // exception object pointer at this point.
    let (ptr_reg, len_reg) = message_scratch_regs(ctx);
    abi::emit_pop_reg_pair(ctx.emitter, ptr_reg, len_reg);
    emit_store_message_fields(ctx, ptr_reg, len_reg);

    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_store_reg_to_symbol(ctx.emitter, result_reg, "_exc_value", 0); // publish the active exception object
    abi::emit_jump(ctx.emitter, "__rt_throw_current"); // enter the standard exception unwinder
    Ok(())
}

/// Moves a two-register pair `(src_a, src_b)` into `(dst_a, dst_b)`, skipping identity moves.
fn move_reg_pair(ctx: &mut FunctionContext<'_>, dst_a: &str, dst_b: &str, src_a: &str, src_b: &str) {
    move_reg(ctx, dst_a, src_a);
    move_reg(ctx, dst_b, src_b);
}

/// Moves one register into another, skipping identity moves. Both targets share `mov dst, src`
/// operand order in this codebase's emitted syntax (AArch64 native, x86_64 Intel-style).
fn move_reg(ctx: &mut FunctionContext<'_>, dst: &str, src: &str) {
    if dst == src {
        return;
    }
    ctx.emitter.instruction(&format!("mov {}, {}", dst, src));                  // shuffle a scratch register ahead of the next runtime call
}

/// Persists the current `__rt_concat` result (in the lhs/output slot) as an owned heap string.
fn emit_persist_from_concat_result(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");              // own a heap copy of the assembled message (x1/x2 in, x1/x2 out)
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, rax");                            // move the message pointer into the persist argument (length already sits in rdx)
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");              // own a heap copy of the assembled message
        }
    }
}

/// Allocates the 32-byte `Throwable` payload and stamps its object header and class id.
fn emit_alloc_type_error_header(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, #32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");               // allocate the TypeError object payload
            ctx.emitter.instruction("mov x9, #6");                              // heap kind 6 = object instance
            ctx.emitter.instruction("str x9, [x0, #-8]");                       // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "x9", "_spl_type_error_class_id", 0); // load TypeError's runtime class id
            ctx.emitter.instruction("str x9, [x0]");                            // store the class id at the object header
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rax, 32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");               // allocate the TypeError object payload
            ctx.emitter.instruction("mov r10, 0x4548504c00000006");             // x86_64 heap-kind word: object magic + kind 6
            ctx.emitter.instruction("mov QWORD PTR [rax - 8], r10");            // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "r10", "_spl_type_error_class_id", 0); // load TypeError's runtime class id
            ctx.emitter.instruction("mov QWORD PTR [rax], r10");                // store the class id at the object header
        }
    }
}

/// Stores the persisted message pointer/length and a zero exception code into the freshly
/// allocated `TypeError` object (whose pointer sits in the target's integer result register).
fn emit_store_message_fields(ctx: &mut FunctionContext<'_>, ptr_reg: &str, len_reg: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("str {}, [x0, #8]", ptr_reg));     // store the exception message pointer
            ctx.emitter.instruction(&format!("str {}, [x0, #16]", len_reg));    // store the exception message length
            ctx.emitter.instruction("str xzr, [x0, #24]");                      // exception code defaults to zero
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("mov QWORD PTR [rax + 8], {}", ptr_reg)); // store the exception message pointer
            ctx.emitter.instruction(&format!("mov QWORD PTR [rax + 16], {}", len_reg)); // store the exception message length
            ctx.emitter.instruction("mov QWORD PTR [rax + 24], 0");             // exception code defaults to zero
        }
    }
}
