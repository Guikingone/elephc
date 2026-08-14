//! Purpose:
//! Shared runtime tag-dispatch `\TypeError` machinery for union-boxed array-family builtin
//! readers. A boxed `Mixed` local produced by an Elvis/ternary join
//! (`$u = $hosts ?: false`) is a CORRECTLY tagged cell (verified via `--emit-ir`: both join arms
//! box through `mixed_box`/`__rt_mixed_from_array_kind` before the merge), so the fix for the
//! family's SIGSEGV belongs at the READ side, not the join — this module is that read-side seam.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::lower_count_dynamic()`, which calls
//!   `emit_mixed_wrong_tag_type_error_dispatch` directly (its accepted-tag set — 4/5/6 for
//!   array/hash/Countable-object — requires its own tag probe before the shared dispatch).
//! - Other gradual builtin and language-operation guards that need the same per-tag diagnostic.
//!
//! Key details:
//! - `emit_mixed_wrong_tag_type_error_dispatch` is the reusable per-tag (int/string/float/
//!   true/false/null/generic-other) dispatch + catchable-`\TypeError`-throw sequence. It mirrors
//!   `array_unshift`'s own dynamic-union wrong-tag dispatch
//!   (`crate::codegen::lower_inst::builtins::arrays::unshift::emit_array_unshift_union_wrong_tag_dispatch`)
//!   but is reusable across every reader in this family instead of being private to one builtin.
//! - Every throw reuses the standard heap-allocation/publish/unwind sequence established by
//!   `crate::codegen::lower_inst::objects::reflection::emit_reflection_class_argument_type_error_throw`
//!   (also mirrored by `unshift`'s own throw builder); the message text is supplied per call site
//!   via a closure so each builtin can match PHP's exact wording (parameter name, position, and
//!   declared type differ per builtin).

use crate::codegen::platform::Arch;
use crate::codegen::context::FunctionContext;

/// Emits the reusable wrong-tag `\TypeError` dispatch for a union-boxed array-family argument.
///
/// Entered via a jump when the argument's post-`__rt_mixed_unbox` runtime tag was not the single
/// accepted tag (4 = indexed array); the tag is still live in the unbox output register
/// (`x0`/`rax`) and the payload low word in `x1`/`rdi` (needed to tell PHP `true` from `false`).
/// Builds and throws a catchable `\TypeError` for whichever concrete PHP type the tag represents,
/// using `message_for(given_type_name)` to build the exact per-builtin wording. Never returns.
pub(in crate::codegen::lower_inst) fn emit_mixed_wrong_tag_type_error_dispatch(
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
    let array_label = ctx.next_label("mixed_array_guard_te_array");
    let object_label = ctx.next_label("mixed_array_guard_te_object");
    let resource_label = ctx.next_label("mixed_array_guard_te_resource");
    let callable_label = ctx.next_label("mixed_array_guard_te_callable");
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
            ctx.emitter.instruction("cmp x0, #4");                              // runtime tag 4 = indexed array
            ctx.emitter.instruction(&format!("b.eq {}", array_label));
            ctx.emitter.instruction("cmp x0, #5");                              // runtime tag 5 = associative array
            ctx.emitter.instruction(&format!("b.eq {}", array_label));
            ctx.emitter.instruction("cmp x0, #6");                              // runtime tag 6 = object
            ctx.emitter.instruction(&format!("b.eq {}", object_label));
            ctx.emitter.instruction("cmp x0, #9");                              // runtime tag 9 = resource
            ctx.emitter.instruction(&format!("b.eq {}", resource_label));
            ctx.emitter.instruction("cmp x0, #10");                             // runtime tag 10 = callable descriptor
            ctx.emitter.instruction(&format!("b.eq {}", callable_label));
            ctx.emitter.instruction(&format!("b {}", generic_label));           // defensive fallback for an unknown runtime tag
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
            ctx.emitter.instruction("cmp rax, 4");                              // runtime tag 4 = indexed array
            ctx.emitter.instruction(&format!("je {}", array_label));
            ctx.emitter.instruction("cmp rax, 5");                              // runtime tag 5 = associative array
            ctx.emitter.instruction(&format!("je {}", array_label));
            ctx.emitter.instruction("cmp rax, 6");                              // runtime tag 6 = object
            ctx.emitter.instruction(&format!("je {}", object_label));
            ctx.emitter.instruction("cmp rax, 9");                              // runtime tag 9 = resource
            ctx.emitter.instruction(&format!("je {}", resource_label));
            ctx.emitter.instruction("cmp rax, 10");                             // runtime tag 10 = callable descriptor
            ctx.emitter.instruction(&format!("je {}", callable_label));
            ctx.emitter.instruction(&format!("jmp {}", generic_label));         // defensive fallback for an unknown runtime tag
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
    emit_type_error_case(ctx, &array_label, "array", message_for);
    emit_type_error_case(ctx, &object_label, "object", message_for);
    emit_type_error_case(ctx, &resource_label, "resource", message_for);
    emit_type_error_case(ctx, &callable_label, "callable", message_for);
    emit_type_error_case(ctx, &generic_label, "mixed", message_for);
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

/// Constructs and throws a catchable `\TypeError` through the canonical exception emitter.
///
/// The shared emitter owns the current throwable layout, object-handle publication, and the
/// uncaught-versus-active-handler split, so gradual array guards cannot drift from other EIR
/// runtime type checks.
pub(in crate::codegen::lower_inst::builtins) fn emit_throw_static_type_error(
    ctx: &mut FunctionContext<'_>,
    message: &str,
) {
    super::super::super::exceptions::emit_type_error(ctx, message);
}
