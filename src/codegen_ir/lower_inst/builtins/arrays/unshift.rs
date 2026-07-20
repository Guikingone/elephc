//! Purpose:
//! Lowers PHP `array_unshift(array &$array, mixed ...$values)` calls for
//! indexed scalar (`Int`/`Bool`) arrays in the EIR backend.
//!
//! Called from:
//! - `crate::codegen_ir::lower_inst::builtins::arrays::lower_array_unshift()`.
//!
//! Key details:
//! - Real variadic support (H5): each of the `N` prepended values is applied
//!   via its own capacity-aware `__rt_array_unshift_grow` call, in REVERSE
//!   source order, so the first-listed value ends up first — php-verified
//!   `array_unshift($a, 1, 2)` → `[1, 2, ...old]`.
//! - COW and capacity growth are handled entirely inside
//!   `__rt_array_unshift_grow` (mirrors `__rt_array_push_int`); this file only
//!   needs to write the (possibly reallocated) array pointer that each call
//!   returns back into the by-ref source local BETWEEN every value, so a
//!   realloc on any iteration keeps aliases (`$b =& $a`) in sync.
//! - Mutates the caller-visible array after copy-on-write splitting.
//! - Returns the new indexed-array length as PHP `int`.
//! - Supports integer and boolean indexed payloads, matching the existing 8-byte helper.
//! - M2 PART B: `lower_array_unshift_union` handles a first argument whose STATIC type is a
//!   union containing an indexed-array member (the `$a = $x ?: false` gradual-typing idiom).
//!   The value is EIR-unwrapped from its boxed Mixed representation with a runtime tag check
//!   instead of the compile-time-known `Array` shape the rest of this file assumes.

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen_ir::context::FunctionContext;
use crate::codegen_ir::{CodegenIrError, Result};
use crate::ir::{Instruction, ValueId};
use crate::types::PhpType;

use super::super::super::{expect_operand, store_if_result};

/// Lowers `array_unshift()` by ensuring uniqueness/capacity, prepending every
/// variadic value (source order preserved), and returning the new count.
pub(super) fn lower_array_unshift(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count_between(inst, "array_unshift", 2, usize::MAX)?;
    let array = expect_operand(inst, 0)?;
    let array_ty = ctx.value_php_type(array)?;
    // A first argument the checker accepted as a UNION containing an indexed-array member (the
    // `$hosts = $x ?: []`-style gradual-typing idiom — see
    // `crate::types::checker::builtins::arrays`'s `array_unshift` branch) is only ever REACHED
    // here as bare `PhpType::Mixed`: `ir_lower` boxes a branch-merged/reassigned value straight
    // into a Mixed cell instead of preserving the original `Union` shape on the EIR value (a
    // declared `array|false`-typed PARAMETER/property is the one case that keeps a genuine
    // `PhpType::Union` — `raw_value_php_type()` would show it — but the checker-accepted call
    // sites this feature targets are all reassignment-merged locals, which arrive here as plain
    // Mixed regardless). Since the checker already rejected any call whose first argument could
    // never resolve to an array (bare bare-`Mixed`/no-array-member unions are NOT accepted by
    // that checker branch), seeing `PhpType::Mixed` here is proof this call passed that gate —
    // safe to route unconditionally into the dynamic runtime-tag-checked path.
    if matches!(array_ty, PhpType::Mixed) {
        return lower_array_unshift_dynamic(ctx, inst, array);
    }
    let elem_ty = array_unshift_element_type(array_ty)?;
    let mut values = Vec::with_capacity(inst.operands.len() - 1);
    for i in 1..inst.operands.len() {
        let value = expect_operand(inst, i)?;
        let value_ty = ctx.value_php_type(value)?.codegen_repr();
        require_array_unshift_value_type(&elem_ty, &value_ty)?;
        values.push(value);
    }
    require_array_unshift_result_type(&inst.result_php_type.codegen_repr())?;

    let source_local = super::source_load_local_slot(ctx, array)?;

    // Prepend in reverse source order: the LAST call places the FIRST-listed
    // value at index 0 (each prepend pushes everything already placed one
    // slot to the right), matching PHP's left-to-right variadic order.
    for &value in values.iter().rev() {
        match ctx.emitter.target.arch {
            Arch::AArch64 => lower_array_unshift_one_aarch64(ctx, array, value)?,
            Arch::X86_64 => lower_array_unshift_one_x86_64(ctx, array, value)?,
        }
        // `__rt_array_unshift_grow` returns the (possibly COW-split and/or
        // reallocated) array pointer in the integer result register — write
        // it back into the SSA value AND the by-ref source local immediately,
        // mirroring the identical pattern `crate::codegen_ir::lower_inst::arrays::lower_array_push`
        // uses for its own (single-value) write-back.
        ctx.store_result_value(array)?;
        if let Some(slot) = source_local {
            ctx.store_value_to_local(slot, array)?;
        }
    }

    emit_load_array_length_as_result(ctx, array)?;
    store_if_result(ctx, inst)
}

/// Returns the supported element payload type for an indexed-array `array_unshift()`.
fn array_unshift_element_type(ty: PhpType) -> Result<PhpType> {
    match ty.codegen_repr() {
        PhpType::Array(elem) => {
            let elem = elem.codegen_repr();
            if matches!(elem, PhpType::Int | PhpType::Bool | PhpType::Void | PhpType::Never) {
                return Ok(elem);
            }
            Err(CodegenIrError::unsupported(format!(
                "array_unshift indexed-array element PHP type {:?}",
                elem
            )))
        }
        other => Err(CodegenIrError::unsupported(format!(
            "array_unshift for PHP type {:?}",
            other
        ))),
    }
}

/// Verifies a prepended value is individually scalar-payload-compatible (`Int`/`Bool`) with the
/// dynamic path's 8-byte runtime slot layout. Unlike `require_array_unshift_value_type` (used by
/// the STATIC path, which cross-checks against a compile-time-known array element type), the
/// dynamic path never learns the target array's declared element type — its own runtime header
/// already tracks that independently (see `__rt_array_unshift_grow`) — so each value only needs
/// to be individually representable in one 8-byte scalar slot.
fn require_array_unshift_dynamic_value_type(value_ty: &PhpType) -> Result<()> {
    if matches!(value_ty, PhpType::Int | PhpType::Bool) {
        return Ok(());
    }
    Err(CodegenIrError::unsupported(format!(
        "array_unshift dynamic-union value PHP type {:?}",
        value_ty
    )))
}

/// Lowers `array_unshift()` when the first argument's codegen-erased type is `PhpType::Mixed`
/// after the checker accepted it as a UNION containing an indexed-array member (the
/// `$hosts = $query['host'] ?? []`-style gradual-typing idiom the Symfony sites use — see
/// `crate::types::checker::builtins::arrays`'s `array_unshift` branch and the dispatch comment in
/// `lower_array_unshift`). The value is represented at runtime as a boxed Mixed cell; this
/// EIR-unwraps it with a runtime tag check: tag 4 (indexed array) proceeds with the SAME
/// per-value `__rt_array_unshift_grow` loop the plain-array path (`lower_array_unshift`) uses,
/// then reboxes the (possibly reallocated) result into a FRESH Mixed cell written back into the
/// ORIGINAL by-ref slot; any other tag throws a catchable, php-verified `\TypeError` (see
/// `emit_array_unshift_union_wrong_tag_dispatch`).
///
/// Ownership/COW: the boxed Mixed cell may be SHARED with another PHP variable (`$b = $a;`
/// copies the cell pointer with a shallow `__rt_incref` on the CELL, not a deep array copy), so
/// the underlying array's OWN refcount can under-count real aliasing introduced purely by
/// cell-sharing. Rather than inspect the cell's own refcount to decide whether a COW split is
/// truly necessary, this UNCONDITIONALLY takes an extra `__rt_incref` on the unboxed array
/// pointer before calling `__rt_array_unshift_grow` — forcing `__rt_array_ensure_unique` to
/// always COW-split away from a value that might be observed through a sibling boxed reference
/// — then releases that synthetic loop-owned reference again once the mutated result is safely
/// wrapped in a brand-new Mixed cell (`__rt_decref_array`), and finally releases the ORIGINAL
/// cell reference the by-ref slot held (`__rt_decref_mixed`, which frees the old cell — and, if
/// it was the sole owner, its stale array payload — once its refcount reaches zero). This trades
/// one guaranteed array clone per call for a simple, branch-free ownership recipe that stays
/// correct regardless of whether the source cell was actually shared.
fn lower_array_unshift_dynamic(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    array: ValueId,
) -> Result<()> {
    let mut values = Vec::with_capacity(inst.operands.len() - 1);
    for i in 1..inst.operands.len() {
        let value = expect_operand(inst, i)?;
        let value_ty = ctx.value_php_type(value)?.codegen_repr();
        require_array_unshift_dynamic_value_type(&value_ty)?;
        values.push(value);
    }
    require_array_unshift_result_type(&inst.result_php_type.codegen_repr())?;

    let source_local = super::source_load_local_slot(ctx, array)?;

    // -- unbox the current Mixed value and gate on the indexed-array runtime tag --
    ctx.load_value_to_result(array)?;                                           // load $array's CURRENT boxed Mixed cell pointer
    let old_cell_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_reserve_temporary_stack(ctx.emitter, 48);                         // [sp+0]=working array ptr, [sp+16]=old cell, [sp+32]=new cell
    abi::emit_store_to_sp(ctx.emitter, old_cell_reg, 16);                       // preserve the old Mixed cell pointer across everything below
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");                      // tag in result reg, payload lo/hi in the string-result reg pair

    let ok_label = ctx.next_label("array_unshift_union_ok");
    let wrong_tag_label = ctx.next_label("array_unshift_union_wrong_tag");
    // `__rt_mixed_unbox`'s OWN output convention (NOT `abi::string_result_regs`, which is a
    // different pointer/length pair used for materialized string VALUES): AArch64 returns
    // tag=x0, payload_lo=x1, payload_hi=x2; x86_64 returns tag=rax, payload_lo=rdi, payload_hi=rdx.
    let payload_lo = match ctx.emitter.target.arch {
        Arch::AArch64 => "x1",
        Arch::X86_64 => "rdi",
    };
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #4");                              // is the boxed union currently holding an indexed array?
            ctx.emitter.instruction(&format!("b.eq {}", ok_label));             // proceed to the in-place unshift loop on a match
            ctx.emitter.instruction(&format!("b {}", wrong_tag_label));         // any other runtime tag is not array-shaped
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 4");                              // is the boxed union currently holding an indexed array?
            ctx.emitter.instruction(&format!("je {}", ok_label));               // proceed to the in-place unshift loop on a match
            ctx.emitter.instruction(&format!("jmp {}", wrong_tag_label));       // any other runtime tag is not array-shaped
        }
    }
    emit_array_unshift_union_wrong_tag_dispatch(ctx, &wrong_tag_label);

    ctx.emitter.label(&ok_label);
    // -- force independent ownership of the unboxed array pointer before mutating --
    // `__rt_incref` (see `crate::codegen::runtime::arrays::incref`) never overwrites the int
    // result register with anything other than the SAME pointer it was given, so the array
    // pointer is still in the result register immediately after the call — safe to persist
    // straight into the working stack slot with no extra reload.
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("mov x0, {}", payload_lo));        // move the unboxed array pointer into the incref argument register
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("mov rax, {}", payload_lo));       // move the unboxed array pointer into the incref argument register
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_incref");                           // synthetic extra owner: forces a COW split below when the cell was shared
    abi::emit_store_to_sp(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);    // [sp+0] = working array pointer

    // Prepend in reverse source order, exactly mirroring the plain-array loop above.
    for &value in values.iter().rev() {
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.load_value_to_reg(value, "x1")?;                            // x1 = the value being prepended
                abi::emit_load_temporary_stack_slot(ctx.emitter, "x0", 0);      // x0 = the current working array pointer
            }
            Arch::X86_64 => {
                ctx.load_value_to_reg(value, "rsi")?;                           // rsi = the value being prepended
                abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", 0);     // rdi = the current working array pointer
            }
        }
        abi::emit_call_label(ctx.emitter, "__rt_array_unshift_grow");           // COW-split/grow/shift/insert; returns the (possibly new) array pointer
        abi::emit_store_to_sp(ctx.emitter, abi::int_result_reg(ctx.emitter), 0); // persist the (possibly reallocated) working array pointer
    }

    // -- rebox the mutated array into a fresh Mixed cell and publish it --
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", 0);          // x1 = final array pointer (mixed_from_value payload lo)
            ctx.emitter.instruction("mov x2, xzr");                             // indexed-array payloads use only the low word
            ctx.emitter.instruction("mov x0, #4");                              // runtime tag 4 = indexed array
        }
        Arch::X86_64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", 0);         // rdi = final array pointer (mixed_from_value payload lo)
            ctx.emitter.instruction("xor rsi, rsi");                            // indexed-array payloads use only the low word
            ctx.emitter.instruction("mov rax, 4");                              // runtime tag 4 = indexed array
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");                 // retains (increfs) the array pointer into a brand-new boxed cell
    abi::emit_store_to_sp(ctx.emitter, abi::int_result_reg(ctx.emitter), 32);   // [sp+32] = new Mixed cell pointer

    // Release the synthetic loop-owned reference to the final array pointer — only the new cell
    // should own it going forward.
    abi::emit_load_temporary_stack_slot(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
    abi::emit_call_label(ctx.emitter, "__rt_decref_array");
    // Release the by-ref slot's reference to the OLD Mixed cell (frees it, and its stale array
    // payload, once nothing else shares it).
    abi::emit_load_temporary_stack_slot(ctx.emitter, abi::int_result_reg(ctx.emitter), 16);
    abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");

    // Publish the new Mixed cell into the SSA value's home and the by-ref local slot.
    abi::emit_load_temporary_stack_slot(ctx.emitter, abi::int_result_reg(ctx.emitter), 32);
    ctx.store_result_value(array)?;
    if let Some(slot) = source_local {
        ctx.store_value_to_local(slot, array)?;
    }

    // Compute the PHP-visible result: the FINAL array's length.
    abi::emit_load_temporary_stack_slot(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction("ldr x0, [x0]"),               // length = the array header's first field
        Arch::X86_64 => ctx.emitter.instruction("mov rax, QWORD PTR [rax]"),    // length = the array header's first field
    }
    abi::emit_release_temporary_stack(ctx.emitter, 48);
    store_if_result(ctx, inst)
}

/// Dispatches on the `__rt_mixed_unbox` result (tag in the int-result register, payload low word
/// in `__rt_mixed_unbox`'s own payload-lo register — both still untouched since the tag-4
/// comparison) to throw a catchable `\TypeError` matching PHP's real `array_unshift(): Argument #1 ($array)
/// must be of type array, X given` wording. php -n VERIFIED for the common scalar/null/bool
/// cases (see `tests/codegen/oop/reflection.rs`-style probes in `tests/codegen/arrays/`):
/// `array_unshift(false, 1)` → `"...false given"`, `array_unshift(null, 1)` →
/// `"...null given"`, `array_unshift("s", 1)` → `"...string given"`. An associative/hash-shaped
/// runtime promotion (tag 5) and compound tags (object/resource/callable/reference-cell) share a
/// generic `"...object given"` fallback rather than PHP's exact per-type wording — a scoped,
/// documented simplification: this compiler's `array_unshift()` runtime helper only understands
/// the flat indexed-array header layout, so a genuinely associative array reaching this dynamic
/// path cannot be soundly mutated in place either way.
fn emit_array_unshift_union_wrong_tag_dispatch(ctx: &mut FunctionContext<'_>, entry_label: &str) {
    let int_label = ctx.next_label("array_unshift_union_te_int");
    let string_label = ctx.next_label("array_unshift_union_te_string");
    let float_label = ctx.next_label("array_unshift_union_te_float");
    let bool_label = ctx.next_label("array_unshift_union_te_bool");
    let true_label = ctx.next_label("array_unshift_union_te_true");
    let false_label = ctx.next_label("array_unshift_union_te_false");
    let null_label = ctx.next_label("array_unshift_union_te_null");
    let generic_label = ctx.next_label("array_unshift_union_te_generic");

    ctx.emitter.label(entry_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #0");                              // runtime tag 0 = int
            ctx.emitter.instruction(&format!("b.eq {}", int_label));
            ctx.emitter.instruction("cmp x0, #1");                              // runtime tag 1 = string
            ctx.emitter.instruction(&format!("b.eq {}", string_label));
            ctx.emitter.instruction("cmp x0, #2");                              // runtime tag 2 = float
            ctx.emitter.instruction(&format!("b.eq {}", float_label));
            ctx.emitter.instruction("cmp x0, #3");                              // runtime tag 3 = bool
            ctx.emitter.instruction(&format!("b.eq {}", bool_label));
            ctx.emitter.instruction("cmp x0, #8");                              // runtime tag 8 = null
            ctx.emitter.instruction(&format!("b.eq {}", null_label));
            ctx.emitter.instruction(&format!("b {}", generic_label));           // array(assoc)/object/resource/callable/other
            ctx.emitter.label(&bool_label);
            ctx.emitter.instruction("cmp x1, #0");                              // is the unboxed boolean payload false?
            ctx.emitter.instruction(&format!("b.eq {}", false_label));
            ctx.emitter.instruction(&format!("b {}", true_label));
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 0");                              // runtime tag 0 = int
            ctx.emitter.instruction(&format!("je {}", int_label));
            ctx.emitter.instruction("cmp rax, 1");                              // runtime tag 1 = string
            ctx.emitter.instruction(&format!("je {}", string_label));
            ctx.emitter.instruction("cmp rax, 2");                              // runtime tag 2 = float
            ctx.emitter.instruction(&format!("je {}", float_label));
            ctx.emitter.instruction("cmp rax, 3");                              // runtime tag 3 = bool
            ctx.emitter.instruction(&format!("je {}", bool_label));
            ctx.emitter.instruction("cmp rax, 8");                              // runtime tag 8 = null
            ctx.emitter.instruction(&format!("je {}", null_label));
            ctx.emitter.instruction(&format!("jmp {}", generic_label));         // array(assoc)/object/resource/callable/other
            ctx.emitter.label(&bool_label);
            ctx.emitter.instruction("test rdi, rdi");                           // is the unboxed boolean payload false?
            ctx.emitter.instruction(&format!("je {}", false_label));
            ctx.emitter.instruction(&format!("jmp {}", true_label));
        }
    }

    emit_array_unshift_union_type_error_case(ctx, &int_label, "int");
    emit_array_unshift_union_type_error_case(ctx, &string_label, "string");
    emit_array_unshift_union_type_error_case(ctx, &float_label, "float");
    emit_array_unshift_union_type_error_case(ctx, &true_label, "true");
    emit_array_unshift_union_type_error_case(ctx, &false_label, "false");
    emit_array_unshift_union_type_error_case(ctx, &null_label, "null");
    emit_array_unshift_union_type_error_case(ctx, &generic_label, "object");
}

/// Emits one concrete `array_unshift()` wrong-tag branch: labels it, then throws a catchable
/// `\TypeError` with PHP's exact message for `given_type`.
fn emit_array_unshift_union_type_error_case(
    ctx: &mut FunctionContext<'_>,
    case_label: &str,
    given_type: &str,
) {
    ctx.emitter.label(case_label);
    let message = format!(
        "array_unshift(): Argument #1 ($array) must be of type array, {} given",
        given_type
    );
    emit_throw_static_type_error(ctx, &message);
}

/// Constructs and throws a catchable `\TypeError` carrying a static `message`, reusing the SAME
/// heap-allocation/publish/unwind sequence the reflection dynamic-dispatch throws use (see
/// `crate::codegen_ir::lower_inst::objects::reflection::emit_reflection_class_argument_type_error_throw`).
fn emit_throw_static_type_error(ctx: &mut FunctionContext<'_>, message: &str) {
    let (message_label, message_len) = ctx.data.add_string(message.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x1", &message_label);       // message pointer
            ctx.emitter
                .instruction(&format!("mov x2, #{}", message_len));            // message byte length
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");             // own a heap copy of the message bytes
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");                  // park the owned message across the allocation call
            ctx.emitter.instruction("mov x0, #32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");              // allocate the TypeError object payload
            ctx.emitter.instruction("mov x9, #6");                              // heap kind 6 = object instance
            ctx.emitter.instruction("str x9, [x0, #-8]");                       // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "x9", "_spl_type_error_class_id", 0); // load TypeError's runtime class id
            ctx.emitter.instruction("str x9, [x0]");                            // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "x9", "x10");                  // reload the owned message pointer/length
            ctx.emitter.instruction("str x9, [x0, #8]");                        // store the exception message pointer
            ctx.emitter.instruction("str x10, [x0, #16]");                      // store the exception message length
            ctx.emitter.instruction("str xzr, [x0, #24]");                      // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "x0", "_exc_value", 0); // publish the active exception object
            abi::emit_jump(ctx.emitter, "__rt_throw_current");                 // enter the standard exception unwinder
        }
        Arch::X86_64 => {
            // `__rt_str_persist`'s REAL x86_64 input convention is rax=ptr, rdx=len (verified
            // against its own body — `crate::codegen::runtime::strings::str_persist`'s stale
            // header comment claims rdi/rdx, but the implementation reads/saves rax/rdx; every
            // OTHER existing x86_64 call site, e.g. `emit_reflection_string_property`, already
            // loads rax/rdx, not rdi/rsi).
            abi::emit_symbol_address(ctx.emitter, "rax", &message_label);      // message pointer
            ctx.emitter
                .instruction(&format!("mov rdx, {}", message_len));            // message byte length
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");             // own a heap copy of the message (ptr in rax, len carried in rdx)
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");                // park the owned message across the allocation call
            ctx.emitter.instruction("mov rax, 32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");              // allocate the TypeError object payload
            ctx.emitter.instruction("mov r10, 0x4548504c00000006");             // x86_64 heap-kind word: object magic + kind 6
            ctx.emitter.instruction("mov QWORD PTR [rax - 8], r10");            // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "r10", "_spl_type_error_class_id", 0); // load TypeError's runtime class id
            ctx.emitter.instruction("mov QWORD PTR [rax], r10");                // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "r10", "r11");                 // reload the owned message pointer/length
            ctx.emitter.instruction("mov QWORD PTR [rax + 8], r10");            // store the exception message pointer
            ctx.emitter.instruction("mov QWORD PTR [rax + 16], r11");           // store the exception message length
            ctx.emitter.instruction("mov QWORD PTR [rax + 24], 0");             // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "rax", "_exc_value", 0); // publish the active exception object
            abi::emit_jump(ctx.emitter, "__rt_throw_current");                 // enter the standard exception unwinder
        }
    }
}

/// Verifies a prepended value matches the runtime helper's scalar slot layout.
fn require_array_unshift_value_type(elem_ty: &PhpType, value_ty: &PhpType) -> Result<()> {
    if matches!(value_ty, PhpType::Int | PhpType::Bool)
        && (elem_ty == value_ty || matches!(elem_ty, PhpType::Void | PhpType::Never))
    {
        return Ok(());
    }
    Err(CodegenIrError::unsupported(format!(
        "array_unshift value PHP type {:?} for indexed-array element PHP type {:?}",
        value_ty,
        elem_ty
    )))
}

/// Verifies the lowered `array_unshift()` result carries PHP's integer count metadata.
fn require_array_unshift_result_type(result_ty: &PhpType) -> Result<()> {
    if result_ty == &PhpType::Int {
        return Ok(());
    }
    Err(CodegenIrError::unsupported(format!(
        "array_unshift result PHP type {:?}",
        result_ty
    )))
}

/// Emits the AArch64 `__rt_array_unshift_grow()` runtime call for one scalar value.
fn lower_array_unshift_one_aarch64(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    value: ValueId,
) -> Result<()> {
    ctx.load_value_to_reg(value, "x1")?;
    ctx.load_value_to_reg(array, "x0")?;
    abi::emit_call_label(ctx.emitter, "__rt_array_unshift_grow");
    Ok(())
}

/// Emits the x86_64 `__rt_array_unshift_grow()` runtime call for one scalar value.
fn lower_array_unshift_one_x86_64(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    value: ValueId,
) -> Result<()> {
    ctx.load_value_to_reg(value, "rsi")?;
    ctx.load_value_to_reg(array, "rdi")?;
    abi::emit_call_label(ctx.emitter, "__rt_array_unshift_grow");
    Ok(())
}

/// Loads the array's current length (header offset 0) into the integer result
/// register — `array_unshift()`'s PHP-visible return value.
fn emit_load_array_length_as_result(ctx: &mut FunctionContext<'_>, array: ValueId) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(array, "x0")?;
            ctx.emitter.instruction("ldr x0, [x0]");                            // length = the array header's first field
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(array, "rax")?;
            ctx.emitter.instruction("mov rax, QWORD PTR [rax]");                // length = the array header's first field
        }
    }
    Ok(())
}
