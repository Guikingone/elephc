//! Purpose:
//! Shared runtime tag-dispatch `\TypeError` machinery for the union-boxed (`array|false`-style
//! gradual-typing idiom) array-family builtin READERS — `implode()`, `array_slice()`, and
//! `count()`/`sizeof()` so far. A boxed `Mixed` local produced by an Elvis/ternary join
//! (`$u = $hosts ?: false`) is a CORRECTLY tagged cell (verified via `--emit-ir`: both join arms
//! box through `mixed_box`/`__rt_mixed_from_array_kind` before the merge), so the fix for the
//! family's SIGSEGV belongs at the READ side, not the join — this module is that read-side seam.
//!
//! Called from:
//! - `crate::codegen_ir::lower_inst::builtins::strings::lower_implode()`'s dynamic Mixed/Union path.
//! - `crate::codegen_ir::lower_inst::builtins::arrays::lower_array_slice()`'s dynamic Mixed path
//!   (`lower_mixed_array_slice_aarch64`/`_x86_64`).
//! - `crate::codegen_ir::lower_inst::builtins::lower_count_dynamic()`, which calls
//!   `emit_mixed_wrong_tag_type_error_dispatch` directly (its accepted-tag set — 4/5/6 for
//!   array/hash/Countable-object — differs from the array-only `emit_borrow_array_or_type_error`
//!   gate, so it runs its own tag probe before falling into the shared dispatch).
//!
//! Key details:
//! - `emit_borrow_array_or_type_error` is the shared unwrap seam `implode`/`array_slice` use: it
//!   unboxes a Mixed cell via `__rt_mixed_unbox`, and on runtime tag 4 (indexed array) leaves the
//!   payload pointer BORROWED (no incref/decref, no ownership transfer, no tag mutation on the
//!   source cell) in the architecture's integer result register (`x0`/`rax`); every other tag
//!   throws through `emit_mixed_wrong_tag_type_error_dispatch`. Because the cell itself is
//!   untouched, COW and the original union local's ownership are unaffected by the call — the
//!   caller must only READ through the returned pointer, never free or mutate it.
//! - `emit_mixed_wrong_tag_type_error_dispatch` is the reusable per-tag (int/string/float/
//!   true/false/null/generic-other) dispatch + catchable-`\TypeError`-throw sequence. It mirrors
//!   `array_unshift`'s own dynamic-union wrong-tag dispatch
//!   (`crate::codegen_ir::lower_inst::builtins::arrays::unshift::emit_array_unshift_union_wrong_tag_dispatch`)
//!   but is reusable across every reader in this family instead of being private to one builtin.
//! - Every throw reuses the standard heap-allocation/publish/unwind sequence established by
//!   `crate::codegen_ir::lower_inst::objects::reflection::emit_reflection_class_argument_type_error_throw`
//!   (also mirrored by `unshift`'s own throw builder); the message text is supplied per call site
//!   via a closure so each builtin can match PHP's exact wording (parameter name, position, and
//!   declared type differ per builtin).

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen_ir::context::FunctionContext;
use crate::codegen_ir::Result;
use crate::ir::ValueId;

/// Runtime tag for an indexed array in the boxed Mixed representation.
const MIXED_TAG_ARRAY: i64 = 4;

/// Emits the shared "borrow an array out of a boxed Mixed cell, or throw" gate.
///
/// Unboxes `mixed_value` via `__rt_mixed_unbox`. On runtime tag 4 (indexed array), leaves the
/// BORROWED payload pointer (the same raw array-header pointer used everywhere else in the
/// runtime) in the architecture's integer result register (`x0`/`rax`) and returns normally. On
/// any other tag, builds and throws a catchable `\TypeError` via
/// `emit_mixed_wrong_tag_type_error_dispatch`, using `message_for(given_type_name)` to build the
/// php-verified per-type message text, and never returns (falls into the shared unwinder).
pub(in crate::codegen_ir::lower_inst::builtins) fn emit_borrow_array_or_type_error(
    ctx: &mut FunctionContext<'_>,
    mixed_value: ValueId,
    message_for: impl Fn(&str) -> String,
) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(mixed_value, "x0")?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");                  // tag=x0, payload_lo=x1, payload_hi=x2
            let ok_label = ctx.next_label("mixed_array_guard_ok");
            let wrong_tag_label = ctx.next_label("mixed_array_guard_wrong_tag");
            ctx.emitter.instruction(&format!("cmp x0, #{}", MIXED_TAG_ARRAY));  // is the boxed union currently holding an indexed array?
            ctx.emitter.instruction(&format!("b.eq {}", ok_label));             // borrow the payload pointer on a match
            ctx.emitter.instruction(&format!("b {}", wrong_tag_label));         // any other runtime tag is not array-shaped
            emit_mixed_wrong_tag_type_error_dispatch(ctx, &wrong_tag_label, &message_for);
            ctx.emitter.label(&ok_label);
            ctx.emitter.instruction("mov x0, x1");                              // leave the borrowed array payload pointer in the result register
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(mixed_value, "rax")?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");                  // tag=rax, payload_lo=rdi, payload_hi=rdx
            let ok_label = ctx.next_label("mixed_array_guard_ok");
            let wrong_tag_label = ctx.next_label("mixed_array_guard_wrong_tag");
            ctx.emitter.instruction(&format!("cmp rax, {}", MIXED_TAG_ARRAY));  // is the boxed union currently holding an indexed array?
            ctx.emitter.instruction(&format!("je {}", ok_label));               // borrow the payload pointer on a match
            ctx.emitter.instruction(&format!("jmp {}", wrong_tag_label));       // any other runtime tag is not array-shaped
            emit_mixed_wrong_tag_type_error_dispatch(ctx, &wrong_tag_label, &message_for);
            ctx.emitter.label(&ok_label);
            ctx.emitter.instruction("mov rax, rdi");                            // leave the borrowed array payload pointer in the result register
        }
    }
    Ok(())
}

/// Emits the reusable wrong-tag `\TypeError` dispatch for a union-boxed array-family argument.
///
/// Entered via a jump when the argument's post-`__rt_mixed_unbox` runtime tag was not the single
/// accepted tag (4 = indexed array); the tag is still live in the unbox output register
/// (`x0`/`rax`) and the payload low word in `x1`/`rdi` (needed to tell PHP `true` from `false`).
/// Builds and throws a catchable `\TypeError` for whichever concrete PHP type the tag represents,
/// using `message_for(given_type_name)` to build the exact per-builtin wording. Never returns.
pub(in crate::codegen_ir::lower_inst::builtins) fn emit_mixed_wrong_tag_type_error_dispatch(
    ctx: &mut FunctionContext<'_>,
    entry_label: &str,
    message_for: &impl Fn(&str) -> String,
) {
    let int_label = ctx.next_label("mixed_array_guard_te_int");
    let string_label = ctx.next_label("mixed_array_guard_te_string");
    let float_label = ctx.next_label("mixed_array_guard_te_float");
    let bool_label = ctx.next_label("mixed_array_guard_te_bool");
    let true_label = ctx.next_label("mixed_array_guard_te_true");
    let false_label = ctx.next_label("mixed_array_guard_te_false");
    let null_label = ctx.next_label("mixed_array_guard_te_null");
    let generic_label = ctx.next_label("mixed_array_guard_te_generic");

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

    emit_type_error_case(ctx, &int_label, "int", message_for);
    emit_type_error_case(ctx, &string_label, "string", message_for);
    emit_type_error_case(ctx, &float_label, "float", message_for);
    emit_type_error_case(ctx, &true_label, "true", message_for);
    emit_type_error_case(ctx, &false_label, "false", message_for);
    emit_type_error_case(ctx, &null_label, "null", message_for);
    emit_type_error_case(ctx, &generic_label, "object", message_for);
}

/// Emits one concrete wrong-tag branch: labels it, builds `message_for(given_type)`, and throws.
fn emit_type_error_case(
    ctx: &mut FunctionContext<'_>,
    case_label: &str,
    given_type: &str,
    message_for: &impl Fn(&str) -> String,
) {
    ctx.emitter.label(case_label);
    let message = message_for(given_type);
    emit_throw_static_type_error(ctx, &message);
}

/// Constructs and throws a catchable `\TypeError` carrying a static `message`, reusing the SAME
/// heap-allocation/publish/unwind sequence
/// `crate::codegen_ir::lower_inst::objects::reflection::emit_reflection_class_argument_type_error_throw`
/// established and `array_unshift`'s dynamic-union path
/// (`crate::codegen_ir::lower_inst::builtins::arrays::unshift`) already reuses.
pub(in crate::codegen_ir::lower_inst::builtins) fn emit_throw_static_type_error(
    ctx: &mut FunctionContext<'_>,
    message: &str,
) {
    let (message_label, message_len) = ctx.data.add_string(message.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x1", &message_label);            // message pointer
            ctx.emitter
                .instruction(&format!("mov x2, #{}", message_len));                 // message byte length
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");                  // own a heap copy of the message bytes
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");                       // park the owned message across the allocation call
            ctx.emitter.instruction("mov x0, #32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");                   // allocate the TypeError object payload
            ctx.emitter.instruction("mov x9, #6");                              // heap kind 6 = object instance
            ctx.emitter.instruction("str x9, [x0, #-8]");                       // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "x9", "_spl_type_error_class_id", 0); // load TypeError's runtime class id
            ctx.emitter.instruction("str x9, [x0]");                            // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "x9", "x10");                       // reload the owned message pointer/length
            ctx.emitter.instruction("str x9, [x0, #8]");                        // store the exception message pointer
            ctx.emitter.instruction("str x10, [x0, #16]");                      // store the exception message length
            ctx.emitter.instruction("str xzr, [x0, #24]");                      // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "x0", "_exc_value", 0);      // publish the active exception object
            abi::emit_jump(ctx.emitter, "__rt_throw_current");                      // enter the standard exception unwinder
        }
        Arch::X86_64 => {
            // `__rt_str_persist`'s REAL x86_64 input convention is rax=ptr, rdx=len (matches every
            // OTHER existing x86_64 call site, e.g. `unshift`'s own throw builder).
            abi::emit_symbol_address(ctx.emitter, "rax", &message_label);           // message pointer
            ctx.emitter
                .instruction(&format!("mov rdx, {}", message_len));                 // message byte length
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");                  // own a heap copy of the message (ptr in rax, len carried in rdx)
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");                     // park the owned message across the allocation call
            ctx.emitter.instruction("mov rax, 32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");                   // allocate the TypeError object payload
            ctx.emitter.instruction("mov r10, 0x4548504c00000006");             // x86_64 heap-kind word: object magic + kind 6
            ctx.emitter.instruction("mov QWORD PTR [rax - 8], r10");            // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "r10", "_spl_type_error_class_id", 0); // load TypeError's runtime class id
            ctx.emitter.instruction("mov QWORD PTR [rax], r10");                // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "r10", "r11");                      // reload the owned message pointer/length
            ctx.emitter.instruction("mov QWORD PTR [rax + 8], r10");            // store the exception message pointer
            ctx.emitter.instruction("mov QWORD PTR [rax + 16], r11");           // store the exception message length
            ctx.emitter.instruction("mov QWORD PTR [rax + 24], 0");             // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "rax", "_exc_value", 0);     // publish the active exception object
            abi::emit_jump(ctx.emitter, "__rt_throw_current");                      // enter the standard exception unwinder
        }
    }
}
