//! Purpose:
//! Lowers class-relation introspection builtins for the EIR backend.
//! Materializes `class_implements()`, `class_parents()`, and `class_uses()`
//! from compile-time class/interface/trait metadata when the target is a
//! literal class-name string, and from the runtime `_class_relation_table`/
//! `_interface_relation_table`/`_trait_relation_table` payload registry
//! (`crate::codegen_support::runtime::data::class_relation_registry`) otherwise: a
//! non-literal string name, or any object argument (always resolved through
//! its runtime class id, never its static declared type — see
//! `emit_dynamic_class_relation_lookup_from_string_result`).
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::lower_language_construct_call()`.
//!
//! Key details:
//! - Results are boxed `Mixed` because PHP returns `array<string,string>|false`.
//! - Associative array results use the shared hash runtime and preserve
//!   `name => name` insertion order.
//! - The literal-string fast path (`optional_const_string_operand` returns
//!   `Some`) is unchanged from before non-literal support was added: it still
//!   materializes the result as a compile-time-unrolled hash, never touching
//!   the runtime relation registry.

use crate::codegen::platform::Arch;
use crate::codegen::{abi, emit_box_current_value_as_mixed};
use crate::codegen::{CodegenIrError, Result};
use crate::ir::{Immediate, Instruction, Op, ValueDef, ValueId};
use crate::names::php_symbol_key;
use crate::types::{ClassInfo, InterfaceInfo, PhpType};

use super::super::super::context::FunctionContext;
use super::{expect_operand, has_eval_context, lower_eval_class_relation, store_if_result};

enum ClassLikeTarget {
    Class(String),
    Interface(String),
    Trait(String),
    Unknown,
}

/// Lowers `class_implements()`, `class_parents()`, and `class_uses()` from static metadata.
pub(crate) fn lower_class_relation(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
) -> Result<()> {
    super::ensure_arg_count_between(inst, name, 1, 2)?;
    let target_value = expect_operand(inst, 0)?;
    if has_eval_context(ctx) {
        return lower_eval_class_relation(ctx, inst, target_value, name);
    }
    let value = target_value;
    match ctx.value_php_type(value)? {
        PhpType::Str => {
            if let Some(raw) = optional_const_string_operand(ctx, value)? {
                return lower_literal_class_relation(ctx, inst, name, &raw);
            }
            // Non-literal string: resolve the target at runtime through the
            // relation registry instead of `ctx.module`'s compile-time tables.
            ctx.load_value_to_result(value)?;
            emit_dynamic_class_relation_lookup_from_string_result(ctx, name);
            store_if_result(ctx, inst)
        }
        PhpType::Object(_) => {
            // Always resolve through the object's RUNTIME class id, never its
            // static declared type: a variable typed as a base class may hold a
            // derived-class instance, and PHP's class_implements()/
            // class_parents()/class_uses() reflect the runtime class (verified
            // against `php -n`: a `Base`-typed parameter holding a `Leaf`
            // instance reports `Leaf`'s interfaces, not `Base`'s).
            ctx.load_value_to_result(value)?;
            super::types::emit_dynamic_object_class_name(ctx, name);
            emit_dynamic_class_relation_lookup_from_string_result(ctx, name);
            store_if_result(ctx, inst)
        }
        PhpType::Mixed | PhpType::Union(_) => {
            emit_mixed_class_relation_lookup(ctx, name, value)?;
            store_if_result(ctx, inst)
        }
        other => Err(CodegenIrError::unsupported(format!(
            "class-relation target PHP type {:?}",
            other
        ))),
    }
}

/// Lowers the literal-string fast path: unchanged compile-time metadata fold.
fn lower_literal_class_relation(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
    raw: &str,
) -> Result<()> {
    let target = resolve_literal_class_like_target(ctx, raw);
    if matches!(target, ClassLikeTarget::Unknown) {
        emit_boxed_bool(ctx, false);
        return store_if_result(ctx, inst);
    }

    let names = relation_names(ctx, name, &target)?;
    emit_string_hash(ctx, &names);
    emit_box_current_value_as_mixed(ctx.emitter, &class_relation_array_type());
    store_if_result(ctx, inst)
}

/// Returns the associative string-set type used by class-relation builtins.
fn class_relation_array_type() -> PhpType {
    PhpType::AssocArray {
        key: Box::new(PhpType::Str),
        value: Box::new(PhpType::Str),
    }
}

/// Emits a boxed boolean result for union-typed class relation fallbacks.
fn emit_boxed_bool(ctx: &mut FunctionContext<'_>, value: bool) {
    abi::emit_load_int_immediate(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        i64::from(value),
    );
    emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Bool);
}

/// Resolves a class-relation target from a literal class-like name.
fn resolve_literal_class_like_target(ctx: &FunctionContext<'_>, raw: &str) -> ClassLikeTarget {
    if let Some(name) = lookup_class_name(ctx, raw) {
        return ClassLikeTarget::Class(name);
    }
    if let Some(name) = lookup_interface_name(ctx, raw) {
        return ClassLikeTarget::Interface(name);
    }
    if let Some(name) = lookup_trait_name(ctx, raw) {
        return ClassLikeTarget::Trait(name);
    }
    ClassLikeTarget::Unknown
}

/// Returns this builtin's fixed payload byte offset within a 64-byte relation
/// row (see `crate::codegen_support::runtime::data::class_relation_registry`).
fn relation_payload_offset(name: &str) -> i64 {
    match name {
        "class_implements" => 16,
        "class_parents" => 32,
        "class_uses" => 48,
        _ => unreachable!("lower_class_relation only dispatches class_implements/parents/uses"),
    }
}

/// Resolves a class-relation target at runtime from a name currently held in
/// the string result registers (`abi::string_result_regs`), searching the
/// `_class_relation_table`/`_interface_relation_table`/`_trait_relation_table`
/// payload registry via `__rt_class_relation_lookup`, then materializes the
/// matched relation list into a boxed `Mixed` array (or `false` on a miss).
fn emit_dynamic_class_relation_lookup_from_string_result(ctx: &mut FunctionContext<'_>, name: &str) {
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    let offset = relation_payload_offset(name);
    let miss_label = ctx.next_label("class_relation_miss");
    let done_label = ctx.next_label("class_relation_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("mov x0, {}", ptr_reg));           // name_ptr -> lookup arg0
            ctx.emitter.instruction(&format!("mov x1, {}", len_reg));           // name_len -> lookup arg1
            ctx.emitter.instruction(&format!("mov x2, #{}", offset));           // this builtin's relation payload offset -> lookup arg2
            ctx.emitter.instruction("bl __rt_class_relation_lookup");           // x0=found, x1=list_ptr, x2=list_count
            ctx.emitter.instruction(&format!("cbz x0, {}", miss_label));        // no relation table has a row for this name
            ctx.emitter.instruction("mov x0, x1");                              // list_ptr -> hash builder arg0
            ctx.emitter.instruction("mov x1, x2");                              // list_count -> hash builder arg1
            ctx.emitter.instruction("bl __rt_hash_from_name_list");             // x0 = the built key===value assoc hash
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("mov rdi, {}", ptr_reg));          // name_ptr -> lookup arg0
            ctx.emitter.instruction(&format!("mov rsi, {}", len_reg));          // name_len -> lookup arg1
            ctx.emitter.instruction(&format!("mov rdx, {}", offset));           // this builtin's relation payload offset -> lookup arg2
            ctx.emitter.instruction("call __rt_class_relation_lookup");         // rax=found, rdi=list_ptr, rsi=list_count
            ctx.emitter.instruction("test rax, rax");                           // did a relation table have a row for this name?
            ctx.emitter.instruction(&format!("jz {}", miss_label));             // no relation table has a row for this name
            ctx.emitter.instruction("call __rt_hash_from_name_list");           // rax = the built key===value assoc hash (rdi/rsi already set)
        }
    }
    emit_box_current_value_as_mixed(ctx.emitter, &class_relation_array_type());
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&miss_label);
    emit_boxed_bool(ctx, false);

    ctx.emitter.label(&done_label);
}

/// Resolves a class-relation target at runtime from a boxed `Mixed`/`Union`
/// value: unboxes it, and dispatches to the object or string runtime lookup
/// path when the payload is an object (tag 6) or a string (tag 1). Any other
/// runtime payload (int, array, bool, ...) has no class-relation meaning in
/// PHP — real `class_implements($int)` throws a `TypeError` — so it degrades
/// to the same `false` result the closed-world registry returns for an
/// unknown class-like name rather than fabricating a plausible-looking array.
fn emit_mixed_class_relation_lookup(
    ctx: &mut FunctionContext<'_>,
    name: &str,
    value: ValueId,
) -> Result<()> {
    let object_label = ctx.next_label("class_relation_mixed_obj");
    let string_label = ctx.next_label("class_relation_mixed_str");
    let done_label = ctx.next_label("class_relation_mixed_done");
    ctx.load_value_to_result(value)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");                  // x0/rax=tag, x1/rdi=lo, x2/rdx=hi
    super::emit_branch_on_gettype_mixed_tag(ctx, 6, &object_label);         // tag 6 = object
    super::emit_branch_on_gettype_mixed_tag(ctx, 1, &string_label);        // tag 1 = string

    // -- neither an object nor a string payload: no relation to report --
    emit_boxed_bool(ctx, false);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&object_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1");                              // unboxed object pointer -> class-name lookup input
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rax, rdi");                            // unboxed object pointer -> class-name lookup input
        }
    }
    super::types::emit_dynamic_object_class_name(ctx, name);
    emit_dynamic_class_relation_lookup_from_string_result(ctx, name);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&string_label);
    if ctx.emitter.target.arch == Arch::X86_64 {
        ctx.emitter.instruction("mov rax, rdi");                                // unboxed string pointer -> string_result_regs' ptr slot
    }
    // AArch64's unboxed (ptr, len) already sit in (x1, x2), the exact
    // `abi::string_result_regs` pair for this target, so no move is needed.
    emit_dynamic_class_relation_lookup_from_string_result(ctx, name);

    ctx.emitter.label(&done_label);
    Ok(())
}

/// Returns the relation names for a known class-like target.
fn relation_names(
    ctx: &FunctionContext<'_>,
    name: &str,
    target: &ClassLikeTarget,
) -> Result<Vec<String>> {
    match name {
        "class_implements" => Ok(class_implements(ctx, target)),
        "class_parents" => Ok(class_parents(ctx, target)),
        "class_uses" => Ok(class_uses(ctx, target)),
        _ => Err(CodegenIrError::unsupported(format!(
            "class-relation builtin {}",
            name
        ))),
    }
}

/// Computes implemented interface names for a class or parent interfaces for an interface.
fn class_implements(ctx: &FunctionContext<'_>, target: &ClassLikeTarget) -> Vec<String> {
    match target {
        ClassLikeTarget::Class(class_name) => lookup_class(ctx, class_name)
            .map(|info| info.interfaces.clone())
            .unwrap_or_default(),
        ClassLikeTarget::Interface(interface_name) => {
            resolve_interface_ancestors(ctx, interface_name)
        }
        ClassLikeTarget::Trait(_) | ClassLikeTarget::Unknown => Vec::new(),
    }
}

/// Computes parent class names from the immediate parent through ancestors.
fn class_parents(ctx: &FunctionContext<'_>, target: &ClassLikeTarget) -> Vec<String> {
    let ClassLikeTarget::Class(class_name) = target else {
        return Vec::new();
    };

    let mut names = Vec::new();
    let mut current = class_name.clone();
    while let Some(info) = lookup_class(ctx, &current) {
        let Some(parent) = &info.parent else {
            break;
        };
        let parent_name = lookup_class_name(ctx, parent).unwrap_or_else(|| parent.clone());
        names.push(parent_name.clone());
        current = parent_name;
    }
    names
}

/// Computes direct trait uses for classes or trait declarations.
fn class_uses(ctx: &FunctionContext<'_>, target: &ClassLikeTarget) -> Vec<String> {
    match target {
        ClassLikeTarget::Class(class_name) => lookup_class(ctx, class_name)
            .map(|info| info.used_traits.clone())
            .unwrap_or_default(),
        ClassLikeTarget::Trait(trait_name) => ctx
            .module
            .declared_trait_uses
            .get(trait_name)
            .cloned()
            .unwrap_or_default(),
        ClassLikeTarget::Interface(_) | ClassLikeTarget::Unknown => Vec::new(),
    }
}

/// Computes `interface_name`'s own transitively extended ancestor interfaces (excluding
/// itself), in PHP's linearization order: this interface's own directly declared `extends`
/// list first (source order, as one contiguous block), then — for each of those parents in
/// that same order — that parent's own ancestor list, individually reversed, appended.
/// Deduplicates by case-insensitive name, keeping the first occurrence.
///
/// `php -n` verified (single-parent chain reverses at every third+ level: `interface JD
/// extends JC extends JB extends JA` reports `[JC, JA, JB]`, not `[JC, JB, JA]`); mirrors
/// `crate::types::checker::schema::classes::interfaces::resolve_interface_ancestors` and
/// `crate::codegen_support::runtime::data::class_relation_registry::collect_interface_parents` —
/// kept in sync by hand since none of the three share this checker-internal/codegen-internal
/// helper directly.
fn resolve_interface_ancestors(ctx: &FunctionContext<'_>, interface_name: &str) -> Vec<String> {
    let Some(interface) = lookup_interface(ctx, interface_name) else {
        return Vec::new();
    };
    if interface.parents.is_empty() {
        return Vec::new();
    }
    let direct_parents: Vec<String> = interface
        .parents
        .iter()
        .map(|parent| lookup_interface_name(ctx, parent).unwrap_or_else(|| parent.clone()))
        .collect();
    let mut names = Vec::new();
    for parent_name in &direct_parents {
        if !names
            .iter()
            .any(|name: &String| php_symbol_key(name) == php_symbol_key(parent_name))
        {
            names.push(parent_name.clone());
        }
    }
    for parent_name in &direct_parents {
        let grandparents = resolve_interface_ancestors(ctx, parent_name);
        for grandparent in grandparents.into_iter().rev() {
            if !names
                .iter()
                .any(|name| php_symbol_key(name) == php_symbol_key(&grandparent))
            {
                names.push(grandparent);
            }
        }
    }
    names
}

/// Allocates and fills an associative string hash in the target result register.
fn emit_string_hash(ctx: &mut FunctionContext<'_>, names: &[String]) {
    let capacity = (names.len() * 2).max(16);
    let value_tag = runtime_str_tag();
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_int_immediate(ctx.emitter, "x0", capacity as i64);
            abi::emit_load_int_immediate(ctx.emitter, "x1", value_tag);
            abi::emit_call_label(ctx.emitter, "__rt_hash_new");
            emit_string_hash_entries_aarch64(ctx, names);
        }
        Arch::X86_64 => {
            abi::emit_load_int_immediate(ctx.emitter, "rdi", capacity as i64);
            abi::emit_load_int_immediate(ctx.emitter, "rsi", value_tag);
            abi::emit_call_label(ctx.emitter, "__rt_hash_new");
            emit_string_hash_entries_x86_64(ctx, names);
        }
    }
}

/// Appends string-set hash entries on AArch64.
fn emit_string_hash_entries_aarch64(ctx: &mut FunctionContext<'_>, names: &[String]) {
    if names.is_empty() {
        return;
    }
    ctx.emitter.instruction("str x0, [sp, #-16]!");                             // park the class-relation hash while inserting metadata entries
    for name in names {
        let (label, len) = ctx.data.add_string(name.as_bytes());
        abi::emit_symbol_address(ctx.emitter, "x1", &label);
        abi::emit_load_int_immediate(ctx.emitter, "x2", len as i64);
        abi::emit_call_label(ctx.emitter, "__rt_hash_normalize_key");
        abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
        abi::emit_symbol_address(ctx.emitter, "x1", &label);
        abi::emit_load_int_immediate(ctx.emitter, "x2", len as i64);
        abi::emit_call_label(ctx.emitter, "__rt_str_persist");
        ctx.emitter.instruction("mov x3, x1");                                  // pass the owned relation name as the hash value pointer
        ctx.emitter.instruction("mov x4, x2");                                  // pass the relation name length as the hash value high word
        abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
        ctx.emitter.instruction("ldr x0, [sp]");                                // reload the current class-relation hash pointer
        abi::emit_load_int_immediate(ctx.emitter, "x5", runtime_str_tag());
        abi::emit_call_label(ctx.emitter, "__rt_hash_set");
        ctx.emitter.instruction("str x0, [sp]");                                // preserve the possibly-grown class-relation hash
    }
    ctx.emitter.instruction("ldr x0, [sp], #16");                               // restore the final class-relation hash as the result
}

/// Appends string-set hash entries on x86_64.
fn emit_string_hash_entries_x86_64(ctx: &mut FunctionContext<'_>, names: &[String]) {
    if names.is_empty() {
        return;
    }
    ctx.emitter.instruction("push rax");                                        // park the class-relation hash while inserting metadata entries
    ctx.emitter.instruction("sub rsp, 8");                                      // keep stack alignment stable across hash helper calls
    for name in names {
        let (label, len) = ctx.data.add_string(name.as_bytes());
        abi::emit_symbol_address(ctx.emitter, "rax", &label);
        abi::emit_load_int_immediate(ctx.emitter, "rdx", len as i64);
        abi::emit_call_label(ctx.emitter, "__rt_hash_normalize_key");
        abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
        abi::emit_symbol_address(ctx.emitter, "rax", &label);
        abi::emit_load_int_immediate(ctx.emitter, "rdx", len as i64);
        abi::emit_call_label(ctx.emitter, "__rt_str_persist");
        ctx.emitter.instruction("mov rcx, rax");                                // pass the owned relation name as the hash value pointer
        ctx.emitter.instruction("mov r8, rdx");                                 // pass the relation name length as the hash value high word
        abi::emit_pop_reg_pair(ctx.emitter, "rsi", "rdx");
        ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 8]");                // reload the current class-relation hash pointer
        abi::emit_load_int_immediate(ctx.emitter, "r9", runtime_str_tag());
        abi::emit_call_label(ctx.emitter, "__rt_hash_set");
        ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rax");                // preserve the possibly-grown class-relation hash
    }
    ctx.emitter.instruction("add rsp, 8");                                      // drop the temporary alignment slot
    ctx.emitter.instruction("pop rax");                                         // restore the final class-relation hash as the result
}

/// Returns the runtime tag for string hash values.
fn runtime_str_tag() -> i64 {
    crate::codegen::runtime_value_tag(&PhpType::Str) as i64
}

/// Looks up a class by PHP-style case-insensitive name.
fn lookup_class<'a>(ctx: &'a FunctionContext<'_>, name: &str) -> Option<&'a ClassInfo> {
    let name = lookup_class_name(ctx, name)?;
    ctx.module.class_infos.get(&name)
}

/// Looks up an interface by PHP-style case-insensitive name.
fn lookup_interface<'a>(
    ctx: &'a FunctionContext<'_>,
    name: &str,
) -> Option<&'a InterfaceInfo> {
    let name = lookup_interface_name(ctx, name)?;
    ctx.module.interface_infos.get(&name)
}

/// Looks up a class name by PHP-style case-insensitive name.
fn lookup_class_name(ctx: &FunctionContext<'_>, raw: &str) -> Option<String> {
    lookup_folded(ctx.module.class_infos.keys(), raw)
}

/// Looks up an interface name by PHP-style case-insensitive name.
fn lookup_interface_name(ctx: &FunctionContext<'_>, raw: &str) -> Option<String> {
    lookup_folded(ctx.module.interface_infos.keys(), raw)
}

/// Looks up a trait name by PHP-style case-insensitive name.
fn lookup_trait_name(ctx: &FunctionContext<'_>, raw: &str) -> Option<String> {
    lookup_folded(ctx.module.trait_table.names.iter(), raw)
}

/// Returns a matching symbol name using PHP case-insensitive comparison.
fn lookup_folded<'a>(names: impl Iterator<Item = &'a String>, raw: &str) -> Option<String> {
    let clean = raw.trim_start_matches('\\');
    let key = php_symbol_key(clean);
    names
        .into_iter()
        .find(|name| php_symbol_key(name.trim_start_matches('\\')) == key)
        .cloned()
}

/// Returns a `ConstStr` operand value, or `None` when the operand is not a literal string.
fn optional_const_string_operand(
    ctx: &FunctionContext<'_>,
    value: ValueId,
) -> Result<Option<String>> {
    let value_ref = ctx
        .function
        .value(value)
        .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))?;
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return Ok(None);
    };
    let inst_ref = ctx
        .function
        .instruction(inst)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
    if inst_ref.op != Op::ConstStr {
        return Ok(None);
    }
    let Some(Immediate::Data(data)) = inst_ref.immediate else {
        return Err(CodegenIrError::invalid_module(
            "string literal operand has no data id",
        ));
    };
    Ok(Some(ctx
        .module
        .data
        .strings
        .get(data.as_raw() as usize)
        .cloned()
        .ok_or_else(|| CodegenIrError::missing_entry("data string", data.as_raw()))?))
}
