//! Purpose:
//! Lowers constant definition, gettype, PHP version, and extension metadata builtins.
//!
//! Called from:
//! - `super` runtime-function dispatch.
//!
//! Key details:
//! - Keeps compile-profile and linked-extension decisions at codegen time, where the effective bridge set is complete.

use super::*;

/// Lowers `define("NAME", value)` with the duplicate-name runtime guard.
pub(crate) fn lower_define(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count(inst, "define", 2)?;
    let name_value = expect_operand(inst, 0)?;
    let value = expect_operand(inst, 1)?;
    let constant_name = const_string_operand(ctx, name_value)?;
    let flag_symbol = ctx.data.add_comm(define_seen_symbol(&constant_name), 8);
    let global_symbol = ir_global_symbol(&constant_name);
    let value_ty = ctx.value_php_type(value)?;
    ctx.data
        .add_comm(global_symbol.clone(), value_ty.codegen_repr().stack_size().max(8));

    let first_label = ctx.next_label("define_first");
    let done_label = ctx.next_label("define_done");
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_load_symbol_to_reg(ctx.emitter, result_reg, &flag_symbol, 0);
    abi::emit_branch_if_int_result_zero(ctx.emitter, &first_label);
    emit_duplicate_define_warning(ctx);
    abi::emit_load_int_immediate(ctx.emitter, result_reg, 0);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&first_label);
    ctx.load_value_to_result(value)?;
    abi::emit_store_result_to_symbol(ctx.emitter, &global_symbol, &value_ty, false);
    abi::emit_load_int_immediate(ctx.emitter, result_reg, 1);
    abi::emit_store_reg_to_symbol(ctx.emitter, result_reg, &flag_symbol, 0);

    ctx.emitter.label(&done_label);
    store_if_result(ctx, inst)
}

/// Emits the PHP warning for a repeated `define()` call.
pub(in crate::codegen::lower_inst) fn emit_duplicate_define_warning(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.adrp("x1", "_diag_define_already_defined_msg");
            ctx.emitter.add_lo12("x1", "x1", "_diag_define_already_defined_msg");
            let length = format!("mov x2, #{}", DEFINE_ALREADY_DEFINED_WARNING.len());
            ctx.emitter.instruction(&length);                                   // pass the duplicate-define warning byte length
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("lea rdi, [rip + _diag_define_already_defined_msg]"); // pass the duplicate-define warning pointer
            let length = format!("mov esi, {}", DEFINE_ALREADY_DEFINED_WARNING.len());
            ctx.emitter.instruction(&length);                                   // pass the duplicate-define warning byte length
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_diag_warning");
}

/// Lowers `gettype(value)` for statically concrete PHP types.
pub(crate) fn lower_gettype(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count(inst, "gettype", 1)?;
    let value = expect_operand(inst, 0)?;
    let ty = ctx.raw_value_php_type(value)?;
    // Dispatch on the codegen representation: a nullable-int union stores an
    // inline tagged scalar, not a boxed Mixed cell, so unboxing it would read
    // a non-pointer payload as a heap cell and crash.
    if matches!(ty.codegen_repr(), PhpType::TaggedScalar) {
        emit_tagged_scalar_gettype(ctx, value)?;
        return store_if_result(ctx, inst);
    }
    if matches!(ty, PhpType::Mixed | PhpType::Union(_)) {
        emit_mixed_gettype(ctx, value)?;
        return store_if_result(ctx, inst);
    }
    if matches!(ty, PhpType::Iterable) {
        emit_iterable_type_name(ctx, value, IterableNaming::Gettype)?;
        return store_if_result(ctx, inst);
    }
    let Some(type_name) = static_gettype_name(&ty) else {
        return Err(CodegenIrError::unsupported(format!(
            "gettype for PHP type {:?}",
            ty
        )));
    };
    emit_type_name_result(ctx, type_name);
    store_if_result(ctx, inst)
}

/// Lowers `get_debug_type(value)` with PHP 8's short scalar names and runtime object class names.
pub(crate) fn lower_get_debug_type(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    ensure_arg_count(inst, "get_debug_type", 1)?;
    let value = expect_operand(inst, 0)?;
    let ty = ctx.raw_value_php_type(value)?;
    if matches!(ty.codegen_repr(), PhpType::TaggedScalar) {
        emit_tagged_scalar_get_debug_type(ctx, value)?;
        return store_if_result(ctx, inst);
    }
    if matches!(ty, PhpType::Mixed | PhpType::Union(_)) {
        emit_mixed_get_debug_type(ctx, value)?;
        return store_if_result(ctx, inst);
    }
    if matches!(ty, PhpType::Object(_)) {
        ctx.load_value_to_result(value)?;
        super::types::emit_dynamic_object_class_name(ctx, "get_class");
        return store_if_result(ctx, inst);
    }
    if matches!(ty, PhpType::Iterable) {
        emit_iterable_type_name(ctx, value, IterableNaming::DebugType)?;
        return store_if_result(ctx, inst);
    }
    // A resource has no compile-time name: PHP prints its DISPLAY type, which an explicit
    // close rewrites in place (`resource (stream)` -> `resource (closed)`), so the name has
    // to be read off the payload at runtime like `get_resource_type()` already does.
    if matches!(ty, PhpType::Resource(_)) {
        ctx.load_value_to_result(value)?;
        abi::emit_call_label(ctx.emitter, "__rt_resource_debug_type_name");
        return store_if_result(ctx, inst);
    }
    let Some(type_name) = static_get_debug_type_name(&ty) else {
        return Err(CodegenIrError::unsupported(format!(
            "get_debug_type for PHP type {:?}",
            ty
        )));
    };
    emit_type_name_result(ctx, type_name);
    store_if_result(ctx, inst)
}

/// Which naming convention an `iterable` type probe answers with.
#[derive(Clone, Copy)]
enum IterableNaming {
    /// `gettype()`: the legacy long names, so `"object"` for a Traversable.
    Gettype,
    /// `get_debug_type()`: PHP 8's names, so the object's own CLASS name.
    DebugType,
}

/// Emits `gettype()` / `get_debug_type()` for a value whose static type is `iterable`.
///
/// `iterable` is `array|Traversable` and, unlike `Mixed`, it is not a boxed cell carrying a tag:
/// the value is a raw heap pointer whose concrete shape lives in the uniform heap header. Naming
/// it therefore means asking `__rt_heap_kind`, the same probe
/// `builtins::type_predicates::emit_iterable_heap_kind_predicate` uses, with heap kind 4 meaning an
/// object instance.
///
/// Answering `array` for every iterable was measured wrong: an `iterable` property holding an
/// `IteratorAggregate` reported `get_debug_type` = `array` while `is_object` on the same value was
/// already answering `true`, so the two disagreed about one value.
fn emit_iterable_type_name(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    naming: IterableNaming,
) -> Result<()> {
    let array_case = ctx.next_label("iterable_type_name_array");
    let done = ctx.next_label("iterable_type_name_done");
    ctx.load_value_to_result(value)?;
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, result_reg);
    abi::emit_call_label(ctx.emitter, "__rt_heap_kind");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #4");                              // heap kind 4 is an object instance
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 4");                              // heap kind 4 is an object instance
        }
    }
    // The pop leaves the flags alone on both targets, so the receiver is back in the result
    // register before either arm needs it and the comparison above still decides the branch.
    abi::emit_pop_reg(ctx.emitter, result_reg);
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction(&format!("b.ne {}", array_case)), // every non-object heap kind an iterable can hold is an array
        Arch::X86_64 => ctx.emitter.instruction(&format!("jne {}", array_case)),   // every non-object heap kind an iterable can hold is an array
    }
    match naming {
        IterableNaming::Gettype => emit_type_name_result(ctx, b"object"),
        IterableNaming::DebugType => super::types::emit_dynamic_object_class_name(ctx, "get_class"),
    }
    abi::emit_jump(ctx.emitter, &done);
    ctx.emitter.label(&array_case);
    emit_type_name_result(ctx, b"array");
    ctx.emitter.label(&done);
    Ok(())
}

/// Emits `get_debug_type()` for an inline tagged scalar containing an integer or null.
fn emit_tagged_scalar_get_debug_type(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
) -> Result<()> {
    let null_case = ctx.next_label("debugtype_tagged_null");
    let done = ctx.next_label("debugtype_tagged_done");
    ctx.load_value_to_result(value)?;
    crate::codegen::sentinels::emit_branch_if_tagged_scalar_null(ctx.emitter, &null_case);
    emit_type_name_result(ctx, b"int");
    abi::emit_jump(ctx.emitter, &done);
    ctx.emitter.label(&null_case);
    emit_type_name_result(ctx, b"null");
    ctx.emitter.label(&done);
    Ok(())
}

/// Emits `get_debug_type()` for a boxed Mixed or Union payload by dispatching on its runtime tag.
fn emit_mixed_get_debug_type(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
) -> Result<()> {
    let integer_case = ctx.next_label("debugtype_mixed_int");
    let double_case = ctx.next_label("debugtype_mixed_float");
    let string_case = ctx.next_label("debugtype_mixed_string");
    let boolean_case = ctx.next_label("debugtype_mixed_bool");
    let null_case = ctx.next_label("debugtype_mixed_null");
    let array_case = ctx.next_label("debugtype_mixed_array");
    let object_case = ctx.next_label("debugtype_mixed_object");
    let resource_case = ctx.next_label("debugtype_mixed_resource");
    let done = ctx.next_label("debugtype_mixed_done");
    ctx.load_value_to_result(value)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    emit_branch_on_gettype_mixed_tag(ctx, 0, &integer_case);
    emit_branch_on_gettype_mixed_tag(ctx, 1, &string_case);
    emit_branch_on_gettype_mixed_tag(ctx, 2, &double_case);
    emit_branch_on_gettype_mixed_tag(ctx, 3, &boolean_case);
    emit_branch_on_gettype_mixed_tag(ctx, 4, &array_case);
    emit_branch_on_gettype_mixed_tag(ctx, 5, &array_case);
    emit_branch_on_gettype_mixed_tag(ctx, 6, &object_case);
    emit_branch_on_gettype_mixed_tag(ctx, 9, &resource_case);
    abi::emit_jump(ctx.emitter, &null_case);

    emit_mixed_gettype_case(ctx, &integer_case, b"int", &done);
    emit_mixed_gettype_case(ctx, &double_case, b"float", &done);
    emit_mixed_gettype_case(ctx, &string_case, b"string", &done);
    emit_mixed_gettype_case(ctx, &boolean_case, b"bool", &done);
    emit_mixed_gettype_case(ctx, &null_case, b"null", &done);
    emit_mixed_gettype_case(ctx, &array_case, b"array", &done);
    emit_mixed_resource_debug_type_case(ctx, &resource_case, &done);

    ctx.emitter.label(&object_case);
    super::types::emit_mixed_object_class_name_from_value(ctx, value, "get_class")?;
    abi::emit_jump(ctx.emitter, &done);
    ctx.emitter.label(&done);
    Ok(())
}

/// Emits the tag-9 arm of `get_debug_type()` on a boxed Mixed: names the resource by its
/// runtime close state instead of a literal.
///
/// The arm is jumped to straight from the tag dispatch, so the unboxed payload low word is
/// still live in the register `__rt_mixed_unbox` left it in; only that word has to move into
/// the helper's input register (`x0` / `rax`).
fn emit_mixed_resource_debug_type_case(
    ctx: &mut FunctionContext<'_>,
    label: &str,
    done: &str,
) {
    ctx.emitter.label(label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1");                              // move the unboxed resource payload into the name resolver's input
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rax, rdi");                            // move the unboxed resource payload into the name resolver's input
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_resource_debug_type_name");
    abi::emit_jump(ctx.emitter, done);
}

/// Returns PHP's `get_debug_type()` spelling for concrete statically known non-object types.
fn static_get_debug_type_name(ty: &PhpType) -> Option<&'static [u8]> {
    match ty {
        PhpType::Int => Some(b"int".as_slice()),
        PhpType::Float => Some(b"float".as_slice()),
        PhpType::Str => Some(b"string".as_slice()),
        PhpType::Bool | PhpType::False => Some(b"bool".as_slice()),
        PhpType::Void | PhpType::Never => Some(b"null".as_slice()),
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Iterable => {
            Some(b"array".as_slice())
        }
        PhpType::Callable => Some(b"Closure".as_slice()),
        // `Resource` is deliberately absent: its name depends on the runtime close state,
        // so `lower_get_debug_type` routes it through `__rt_resource_debug_type_name`
        // before reaching here. Answering a literal would print `resource`, a name PHP
        // never gives a resource.
        _ => None,
    }
}

/// Emits `gettype()` for an inline tagged scalar by dispatching on its tag word.
pub(in crate::codegen::lower_inst) fn emit_tagged_scalar_gettype(ctx: &mut FunctionContext<'_>, value: ValueId) -> Result<()> {
    let null_case = ctx.next_label("gettype_tagged_null");
    let done = ctx.next_label("gettype_tagged_done");
    ctx.load_value_to_result(value)?;
    crate::codegen::sentinels::emit_branch_if_tagged_scalar_null(ctx.emitter, &null_case);
    emit_type_name_result(ctx, b"integer");
    abi::emit_jump(ctx.emitter, &done);
    ctx.emitter.label(&null_case);
    emit_type_name_result(ctx, b"NULL");
    ctx.emitter.label(&done);
    Ok(())
}

/// Emits `gettype()` for a boxed Mixed or Union payload by dispatching on runtime tags.
pub(in crate::codegen::lower_inst) fn emit_mixed_gettype(ctx: &mut FunctionContext<'_>, value: ValueId) -> Result<()> {
    let integer_case = ctx.next_label("gettype_mixed_integer");
    let double_case = ctx.next_label("gettype_mixed_double");
    let string_case = ctx.next_label("gettype_mixed_string");
    let boolean_case = ctx.next_label("gettype_mixed_boolean");
    let null_case = ctx.next_label("gettype_mixed_null");
    let array_case = ctx.next_label("gettype_mixed_array");
    let object_case = ctx.next_label("gettype_mixed_object");
    let resource_case = ctx.next_label("gettype_mixed_resource");
    let done = ctx.next_label("gettype_mixed_done");
    ctx.load_value_to_result(value)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    emit_branch_on_gettype_mixed_tag(ctx, 0, &integer_case);
    emit_branch_on_gettype_mixed_tag(ctx, 1, &string_case);
    emit_branch_on_gettype_mixed_tag(ctx, 2, &double_case);
    emit_branch_on_gettype_mixed_tag(ctx, 3, &boolean_case);
    emit_branch_on_gettype_mixed_tag(ctx, 4, &array_case);
    emit_branch_on_gettype_mixed_tag(ctx, 5, &array_case);
    emit_branch_on_gettype_mixed_tag(ctx, 6, &object_case);
    emit_branch_on_gettype_mixed_tag(ctx, 10, &object_case);
    emit_branch_on_gettype_mixed_tag(ctx, 9, &resource_case);
    abi::emit_jump(ctx.emitter, &null_case);

    emit_mixed_gettype_case(ctx, &integer_case, b"integer", &done);
    emit_mixed_gettype_case(ctx, &double_case, b"double", &done);
    emit_mixed_gettype_case(ctx, &string_case, b"string", &done);
    emit_mixed_gettype_case(ctx, &boolean_case, b"boolean", &done);
    emit_mixed_gettype_case(ctx, &null_case, b"NULL", &done);
    emit_mixed_gettype_case(ctx, &array_case, b"array", &done);
    emit_mixed_gettype_case(ctx, &object_case, b"object", &done);
    emit_mixed_gettype_case(ctx, &resource_case, b"resource", &done);
    ctx.emitter.label(&done);
    Ok(())
}

/// Branches to a `gettype()` case when the unboxed Mixed runtime tag matches.
pub(in crate::codegen::lower_inst) fn emit_branch_on_gettype_mixed_tag(ctx: &mut FunctionContext<'_>, tag: u8, label: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cmp x0, #{}", tag));              // compare the unboxed Mixed tag against this gettype() case
            ctx.emitter.instruction(&format!("b.eq {}", label));                // branch to the matching gettype() type-name case
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("cmp rax, {}", tag));              // compare the unboxed Mixed tag against this gettype() case
            ctx.emitter.instruction(&format!("je {}", label));                  // branch to the matching gettype() type-name case
        }
    }
}

/// Selects one static PHP type-name string and rejoins the `gettype()` dispatch.
pub(in crate::codegen::lower_inst) fn emit_mixed_gettype_case(ctx: &mut FunctionContext<'_>, label: &str, type_name: &[u8], done: &str) {
    ctx.emitter.label(label);
    emit_type_name_result(ctx, type_name);
    abi::emit_jump(ctx.emitter, done);
}

/// Returns PHP's `gettype()` spelling for concrete statically known types.
pub(in crate::codegen::lower_inst) fn static_gettype_name(ty: &PhpType) -> Option<&'static [u8]> {
    match ty {
        PhpType::Int => Some(b"integer".as_slice()),
        PhpType::Float => Some(b"double".as_slice()),
        PhpType::Str => Some(b"string".as_slice()),
        PhpType::Bool | PhpType::False => Some(b"boolean".as_slice()),
        PhpType::Void | PhpType::Never => Some(b"NULL".as_slice()),
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Iterable => {
            Some(b"array".as_slice())
        }
        PhpType::Callable => Some(b"object".as_slice()),
        PhpType::Object(_) => Some(b"object".as_slice()),
        PhpType::Pointer(_) => Some(b"pointer".as_slice()),
        PhpType::Buffer(_) => Some(b"buffer".as_slice()),
        PhpType::Packed(_) => Some(b"packed".as_slice()),
        PhpType::Resource(_) => Some(b"resource".as_slice()),
        PhpType::Mixed | PhpType::Union(_) | PhpType::TaggedScalar => None,
    }
}

/// Emits a static PHP type-name string into the target string result registers.
pub(in crate::codegen::lower_inst) fn emit_type_name_result(ctx: &mut FunctionContext<'_>, type_name: &[u8]) {
    let (label, len) = ctx.data.add_string(type_name);
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    abi::emit_symbol_address(ctx.emitter, ptr_reg, &label);
    abi::emit_load_int_immediate(ctx.emitter, len_reg, len as i64);
}

/// Emits the reported PHP version string into the string result registers.
///
/// The value is `crate::web_prelude::PhpVersion::version_string()` for the compilation's
/// `--php-version` profile — the PHP LANGUAGE version elephc targets, never elephc's own
/// package version. Reference PHP 8.5.6 reports `8.5.6` here; elephc reports `8.5.0`, the
/// same deliberate `.0` divergence `opcache_get_configuration()['version']['version']` already
/// makes (see `PhpVersion::version_string` for the rule).
pub(in crate::codegen::lower_inst) fn emit_reported_php_version(ctx: &mut FunctionContext<'_>) {
    let version = crate::codegen::compile_php_version().version_string();
    let (label, len) = ctx.data.add_string(version.as_bytes());
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    abi::emit_symbol_address(ctx.emitter, ptr_reg, &label);
    abi::emit_load_int_immediate(ctx.emitter, len_reg, len as i64);
}

/// Lowers `phpversion()` and `phpversion($extension)`.
///
/// - `phpversion()` folds to a static string (result type `Str`).
/// - `phpversion($extension)` folds to `string|false`, which the backend represents as a
///   boxed `Mixed` cell (result type `Mixed`, matching the builtin's `eir_result_type`). A
///   loaded extension yields the SAME version string as `phpversion()` — reference PHP reports
///   the interpreter's own version for every bundled extension (verified on 8.5.6: `Core`,
///   `json`, `pcre`, `Zend OPcache`, … all report `8.5.6`), and every extension elephc reports
///   as loaded is a bundled-equivalent, so there is no per-extension version to track. An
///   unknown extension yields `false`.
///
/// MEMBERSHIP IS `extension_loaded()`'s, EXACTLY: both the literal fold and the dynamic
/// membership test go through [`extension_is_loaded`] / [`dynamic_extension_loaded_candidates`],
/// so `phpversion($e) !== false` and `extension_loaded($e)` cannot disagree — including for the
/// bridges linked into this particular compilation.
///
/// WHY THE DECISION LIVES HERE AND NOT IN THE CHECKER: the effective extension set is
/// core ∪ `crate::codegen::linked_extensions()`, and the linked set is only complete after type
/// checking (it includes bridges auto-detected from `check_result.required_libraries`). A
/// checker-side or EIR-side fold would therefore see a SMALLER set than this one and could
/// answer `false` where codegen answers a string. Typing the whole one-argument arity as
/// `string|false` keeps the single decision point here, where the set is truthful.
pub(crate) fn lower_phpversion(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count_between(inst, "phpversion", 0, 1)?;
    let Some(value) = inst.operands.first().copied() else {
        emit_reported_php_version(ctx);
        return store_if_result(ctx, inst);
    };
    if let Some(extension_name) = maybe_const_string_operand(ctx, value)? {
        if extension_is_loaded(&extension_name) {
            emit_reported_php_version(ctx);
            crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Str);
        } else {
            emit_static_bool(ctx, false);
            crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Bool);
        }
    } else {
        lower_dynamic_phpversion(ctx, value)?;
    }
    store_if_result(ctx, inst)
}

/// Lowers a dynamic (non-literal) `phpversion($name)` over the effective extension set.
///
/// Same shape as [`lower_dynamic_extension_loaded`] — the runtime string is materialized into
/// one 16-byte temporary stack slot and compared against every baked candidate with
/// `__rt_strcasecmp`, matching PHP's case-insensitive extension-name lookup (verified on
/// reference 8.5.6: `phpversion('core')` and `phpversion('Core')` both return the version).
/// Only the *answer* differs: a match yields the boxed version string, a miss yields boxed
/// `false`.
///
/// The temporary slot must be released on BOTH exits, and the boxing happens after the
/// branches rejoin only for the matched arm, because the two arms box different tags. Each arm
/// therefore boxes its own value and jumps to the shared release.
pub(in crate::codegen::lower_inst) fn lower_dynamic_phpversion(ctx: &mut FunctionContext<'_>, value: ValueId) -> Result<()> {
    if ctx.value_php_type(value)?.codegen_repr() != PhpType::Str {
        return Err(CodegenIrError::unsupported(
            "phpversion with non-string dynamic extension name",
        ));
    }
    let candidates = dynamic_extension_loaded_candidates();
    if candidates.is_empty() {
        emit_static_bool(ctx, false);
        crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Bool);
        return Ok(());
    }

    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    ctx.load_string_value_to_regs(value, ptr_reg, len_reg)?;
    abi::emit_push_reg_pair(ctx.emitter, ptr_reg, len_reg);

    let matched_label = ctx.next_label("phpversion_dynamic_match");
    let done_label = ctx.next_label("phpversion_dynamic_done");
    for candidate in &candidates {
        emit_branch_if_saved_string_matches_ci(ctx, candidate.as_bytes(), &matched_label);
    }
    emit_static_bool(ctx, false);
    crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Bool);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&matched_label);
    emit_reported_php_version(ctx);
    crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Str);

    ctx.emitter.label(&done_label);
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    Ok(())
}

/// Lowers `defined("NAME")` for compile-time string constant names.
pub(crate) fn lower_defined(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count(inst, "defined", 1)?;
    let value = expect_operand(inst, 0)?;
    if let Some(constant_name) = maybe_const_string_operand(ctx, value)? {
        emit_static_bool(
            ctx,
            ctx.has_global_name(&constant_name) || const_registry_contains(ctx, &constant_name),
        );
    } else {
        emit_registry_string_lookup(ctx, value, "defined", "__rt_defined")?;
    }
    store_if_result(ctx, inst)
}

/// Lowers `constant($name)` against the closed-world scalar constant registry.
pub(crate) fn lower_constant(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count(inst, "constant", 1)?;
    let value = expect_operand(inst, 0)?;
    emit_registry_string_lookup(ctx, value, "constant", "__rt_constant")?;
    store_if_result(ctx, inst)
}

/// Returns whether the emitted constant registry contains a canonicalized global name.
fn const_registry_contains(ctx: &FunctionContext<'_>, name: &str) -> bool {
    let normalized = name.trim_start_matches('\\');
    ctx.module.global_constants.contains_key(normalized)
}

/// Materializes a string constant name and invokes a runtime registry lookup helper.
fn emit_registry_string_lookup(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    builtin_name: &str,
    helper: &str,
) -> Result<()> {
    let ty = ctx.load_value_to_result(value)?;
    match ty.codegen_repr() {
        PhpType::Str => {}
        PhpType::Mixed | PhpType::Union(_) => {
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_string");
        }
        other => {
            return Err(CodegenIrError::unsupported(format!(
                "{} name with PHP type {:?}",
                builtin_name, other
            )))
        }
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1");                              // move string pointer into helper argument 0
            ctx.emitter.instruction("mov x1, x2");                              // move string length into helper argument 1
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, rax");                            // move string pointer into helper argument 0
            ctx.emitter.instruction("mov rsi, rdx");                            // move string length into helper argument 1
        }
    }
    abi::emit_call_label(ctx.emitter, helper);
    Ok(())
}

/// Compile-time-known set of "loaded" PHP extensions for `extension_loaded()` and the regular
/// (non-Zend) list returned by `get_loaded_extensions(false)`.
///
/// KEEP IN SYNC with `crates/elephc-magician/src/interpreter/builtins/network_env/extension_loaded.rs`.
/// This is only the always-present core set. Extensions this compilation actually provides
/// (e.g. `hash`, `openssl` from linked bridges, and the injected PHP surfaces `PDO` / `mysqli`
/// riding the shared `elephc_pdo` archive) are added on top per-compilation via
/// `crate::codegen::linked_extensions()` — see [`extension_is_loaded`] and
/// `lower_get_loaded_extensions`. Magician mirrors this set but always adds `bcmath`, which it
/// implements directly; AOT adds `bcmath` only when `elephc_bcmath` is linked. Other bridge-linked
/// extensions and injected surfaces such as `PDO` and `mysqli` remain absent in eval because it
/// has no AOT link manifest (documented divergence: `extension_loaded('PDO')` and
/// `extension_loaded('mysqli')` are `false` in eval).
pub(crate) const CORE_LOADED_EXTENSIONS: &[&str] = &[
    "Core",
    "standard",
    "SPL",
    "json",
    "pcre",
    "date",
    "ctype",
    "mbstring",
    "Reflection",
    "Zend OPcache",
];

/// Compile-time-known set of loaded Zend extensions returned by `get_loaded_extensions(true)`.
///
/// Elephc ships the OPcache Zend extension but no Xdebug, so this list holds only "Zend OPcache".
/// KEEP IN SYNC with `crates/elephc-magician/src/interpreter/builtins/network_env/get_loaded_extensions.rs`.
pub(crate) const ZEND_LOADED_EXTENSIONS: &[&str] = &["Zend OPcache"];

/// Returns true when `name` matches an always-present core extension OR an extension
/// this compilation actually provides (`crate::codegen::linked_extensions()`),
/// compared case-insensitively.
///
/// Mirrors PHP's case-insensitive extension-name comparison: only the canonical names match
/// (e.g. "opcache" is not an alias for "Zend OPcache"). The linked set is populated by the
/// pipeline before codegen from the bridges this program links (e.g. `hash` when a program
/// uses `hash()`) plus the injected PHP surfaces (`PDO` under `--with-pdo` or detected PDO
/// usage, `mysqli` for the mysqli surface — both ride the same `elephc_pdo` archive, so the
/// archive alone identifies neither), so a bridge-free program reports only the core set.
pub(in crate::codegen::lower_inst) fn extension_is_loaded(name: &str) -> bool {
    CORE_LOADED_EXTENSIONS
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(name))
        || crate::codegen::linked_extensions()
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(name))
}

/// Lowers `extension_loaded($extension)` over the effective extension set.
///
/// A literal name const-folds to a static boolean (no runtime cost). A dynamic name is lowered
/// to a case-insensitive membership test against the same effective set (core ∪ linked bridges),
/// which is baked into the binary at compile time — see [`lower_dynamic_extension_loaded`].
pub(crate) fn lower_extension_loaded(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count(inst, "extension_loaded", 1)?;
    let value = expect_operand(inst, 0)?;
    if let Some(extension_name) = maybe_const_string_operand(ctx, value)? {
        emit_static_bool(ctx, extension_is_loaded(&extension_name));
    } else {
        lower_dynamic_extension_loaded(ctx, value)?;
    }
    store_if_result(ctx, inst)
}

/// Lowers a dynamic (non-literal) `extension_loaded($name)` membership test.
///
/// The extension set is fully known at compile time (core ∪ the bridges this program links), so
/// only the *name* is dynamic. The runtime string is materialized into one 16-byte temporary
/// stack slot (pointer + length) and compared against every baked candidate with
/// `__rt_strcasecmp`, matching PHP's case-insensitive extension-name lookup
/// (`extension_loaded('JSON') === extension_loaded('json')`).
///
/// The temporary slot is read back from SP before each comparison rather than being kept in a
/// register, because `__rt_strcasecmp` receives its arguments in caller-saved registers. The slot
/// itself survives every call: `__rt_strcasecmp` is a leaf helper that never touches SP.
pub(in crate::codegen::lower_inst) fn lower_dynamic_extension_loaded(ctx: &mut FunctionContext<'_>, value: ValueId) -> Result<()> {
    if ctx.value_php_type(value)?.codegen_repr() != PhpType::Str {
        return Err(CodegenIrError::unsupported(
            "extension_loaded with non-string dynamic name",
        ));
    }
    let candidates = dynamic_extension_loaded_candidates();
    if candidates.is_empty() {
        emit_static_bool(ctx, false);
        return Ok(());
    }

    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    ctx.load_string_value_to_regs(value, ptr_reg, len_reg)?;
    abi::emit_push_reg_pair(ctx.emitter, ptr_reg, len_reg);

    let matched_label = ctx.next_label("extension_loaded_dynamic_match");
    let done_label = ctx.next_label("extension_loaded_dynamic_done");
    for candidate in &candidates {
        emit_branch_if_saved_string_matches_ci(ctx, candidate.as_bytes(), &matched_label);
    }
    emit_static_bool(ctx, false);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&matched_label);
    emit_static_bool(ctx, true);

    ctx.emitter.label(&done_label);
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    Ok(())
}

/// Returns the effective extension set a dynamic `extension_loaded()` test compares against.
///
/// This is exactly the set [`extension_is_loaded`] folds against for a literal name: the
/// always-present core extensions plus the canonical names of the bridges actually linked into
/// this compilation, de-duplicated case-insensitively so a bridge that shadows a core name is not
/// compared twice.
pub(in crate::codegen::lower_inst) fn dynamic_extension_loaded_candidates() -> Vec<String> {
    let mut candidates: Vec<String> = CORE_LOADED_EXTENSIONS
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    for extension in crate::codegen::linked_extensions() {
        if !candidates
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(&extension))
        {
            candidates.push(extension);
        }
    }
    candidates
}
