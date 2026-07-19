//! Purpose:
//! Lowers `get_class_methods(object|string): array` for the EIR backend:
//! calling-scope method-name visibility (public-only from outside a class,
//! public+protected+own-private from inside a matching method), in PHP
//! declaration order.
//!
//! Called from:
//! - `crate::codegen_ir::lower_inst::builtins::lower_builtin_call()`.
//!
//! Key details:
//! - COMPILE-TIME BAKE, matching `ReflectionClass::getMethods()`'s own strategy
//!   (`crate::codegen_ir::lower_inst::objects::reflection`): the target class
//!   AND the calling scope are both resolved to compile-time-known names
//!   before this ever emits assembly, so the result is a plain compile-time
//!   `Vec<String>` materialized as an indexed string array — no runtime table
//!   walk needed. Rides the K1 `decl_order` semantics (declaration-order
//!   collapsing: an override keeps the position of the level that
//!   (re)declares it) by mirroring
//!   `crate::codegen::runtime::data::method_decl_order_and_names`'s ancestor
//!   walk — READ-ONLY on `crate::codegen::runtime::data::reflect_member_registry`/
//!   `crate::codegen_ir::lower_inst::objects::reflection_members` (K1-owned,
//!   not edited here); this file owns its OWN small duplicate of the walk
//!   because it needs a DIFFERENT exposure filter than `ReflectionClass`'s
//!   (calling-scope visibility, not reflection's "everything but
//!   ancestor-private").
//! - Scope: only a literal class-name string, or an object argument with a
//!   concrete STATIC `PhpType::Object(name)` type, are supported — resolved
//!   against the argument's DECLARED type, not its (potentially more derived)
//!   runtime type. A `Base`-typed variable holding a `Leaf` instance reports
//!   `Base`'s methods, not `Leaf`'s (documented residual — unlike
//!   `class_implements()`/`class_parents()`, which resolve through the
//!   runtime class id). Mixed/Union arguments and non-literal strings are a
//!   loud `CodegenIrError::unsupported`, never a silent guess.
//! - "Calling scope" is the ENCLOSING class of the current EIR function
//!   (`FunctionContext::function.name`'s `"Class::method"` prefix, mirroring
//!   `crate::codegen_ir::mod::current_function_class`) when it matches the
//!   target class exactly (case-insensitively). Any other enclosing class (a
//!   different class in the same hierarchy) falls back to the public-only
//!   filter — never over-accepted as if it were self scope (php -n verified:
//!   only the EXACT declaring/receiving class's own scope sees non-public
//!   members via `get_class_methods`, not siblings/ancestors in general).

use std::collections::HashSet;

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen_ir::{CodegenIrError, Result};
use crate::ir::{Immediate, Instruction, Op, ValueDef, ValueId};
use crate::names::php_symbol_key;
use crate::parser::ast::Visibility;
use crate::types::PhpType;

use super::super::super::context::FunctionContext;
use super::{expect_operand, store_if_result};

/// Lowers `get_class_methods(object|string): array<string>`.
pub(super) fn lower_get_class_methods(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::ensure_arg_count(inst, "get_class_methods", 1)?;
    let arg = expect_operand(inst, 0)?;
    let target_class = resolve_target_class(ctx, arg)?;
    let caller_class = current_function_class(ctx);
    let self_scope = caller_class
        .map(|c| php_symbol_key(c) == php_symbol_key(&target_class))
        .unwrap_or(false);
    let names = get_class_methods_names(ctx, &target_class, self_scope);
    emit_string_array(ctx, &names)?;
    store_if_result(ctx, inst)
}

/// Resolves the operand to a compile-time-known class name: a literal string,
/// or an argument with a concrete static `PhpType::Object(name)` type. Any
/// other shape is a loud, disclosed unsupported error (see the file-level doc
/// comment).
fn resolve_target_class(ctx: &mut FunctionContext<'_>, value: ValueId) -> Result<String> {
    if let Some(literal) = optional_const_string_operand(ctx, value)? {
        let clean = literal.trim_start_matches('\\');
        return lookup_class_name(ctx, clean).ok_or_else(|| {
            CodegenIrError::unsupported(format!(
                "get_class_methods(): class \"{}\" is not declared",
                clean
            ))
        });
    }
    match ctx.value_php_type(value)? {
        PhpType::Object(name) => lookup_class_name(ctx, &name).ok_or_else(|| {
            CodegenIrError::unsupported(format!(
                "get_class_methods(): class \"{}\" is not declared",
                name
            ))
        }),
        other => Err(CodegenIrError::unsupported(format!(
            "get_class_methods() requires a literal class-name string or an object of statically-known type in AOT mode (got {:?})",
            other
        ))),
    }
}

/// Returns the class encoded in the CURRENT EIR function's `"Class::method"`
/// name, or `None` for a free function/top-level scope. Mirrors
/// `crate::codegen_ir::mod::current_function_class` (private to that module).
fn current_function_class<'a>(ctx: &'a FunctionContext<'_>) -> Option<&'a str> {
    ctx.function.name.rsplit_once("::").map(|(class_name, _)| class_name)
}

/// Looks up a class by PHP-style case-insensitive name (mirrors
/// `crate::codegen_ir::lower_inst::builtins::class_relations::lookup_class_name`,
/// duplicated here per this directory's established precedent for small
/// single-use lookups).
fn lookup_class_name(ctx: &FunctionContext<'_>, raw: &str) -> Option<String> {
    let key = php_symbol_key(raw);
    ctx.module
        .class_infos
        .keys()
        .find(|name| php_symbol_key(name) == key)
        .cloned()
}

/// Walks `class_name`'s ancestor chain (itself, then each `parent`, …) and
/// returns the PHP `get_class_methods()`-visible method names in PHP's real
/// declaration order: `class_name`'s own declared methods first (their own
/// source order), then each ancestor's own declared methods appended (nearest
/// first), skipping any key already claimed by a more-derived level (an
/// override keeps the position AND the visibility of the level that
/// (re)declares it — matches
/// `crate::codegen::runtime::data::method_decl_order_and_names`'s decl_order
/// semantics, php -n verified).
///
/// `self_scope` selects the exposure filter:
/// - `true` (calling from inside a method of `class_name` itself): a name is
///   visible unless it is PRIVATE and declared by a stricter ancestor (own
///   private methods and any protected/public method, inherited or not, are
///   visible) — php -n verified against a 2-level hierarchy.
/// - `false` (anywhere else, including a DIFFERENT enclosing class): only
///   PUBLIC methods are visible — php -n verified: calling `get_class_methods`
///   from outside any class, or from an unrelated class, sees public members
///   only.
fn get_class_methods_names(ctx: &FunctionContext<'_>, class_name: &str, self_scope: bool) -> Vec<String> {
    let mut claimed: HashSet<String> = HashSet::new();
    let mut names = Vec::new();
    let mut current = Some(class_name.to_string());
    let mut visited = HashSet::new();
    while let Some(level_name) = current {
        if !visited.insert(level_name.clone()) {
            break; // cycle guard against malformed metadata
        }
        let Some(info) = ctx.module.class_infos.get(&level_name) else {
            break;
        };
        for decl in &info.method_decls {
            let key = php_symbol_key(&decl.name);
            if claimed.contains(&key) {
                continue; // already claimed by a more-derived level
            }
            claimed.insert(key);
            let visible = if self_scope {
                decl.visibility != Visibility::Private || level_name == class_name
            } else {
                decl.visibility == Visibility::Public
            };
            if visible {
                names.push(decl.name.clone());
            }
        }
        current = info.parent.clone();
    }
    names
}

/// Returns a `ConstStr` operand value, or `None` when the operand is not a
/// literal string (mirrors `class_relations::optional_const_string_operand`).
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

/// Allocates an indexed string array and appends every method name (mirrors
/// `builtins::types::emit_string_array`, duplicated here — that function is
/// private to `types.rs`).
fn emit_string_array(ctx: &mut FunctionContext<'_>, names: &[String]) -> Result<()> {
    let capacity = names.len().max(1);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_int_immediate(ctx.emitter, "x0", capacity as i64);
            abi::emit_load_int_immediate(ctx.emitter, "x1", 16);
        }
        Arch::X86_64 => {
            abi::emit_load_int_immediate(ctx.emitter, "rdi", capacity as i64);
            abi::emit_load_int_immediate(ctx.emitter, "rsi", 16);
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_array_new");
    if names.is_empty() {
        return Ok(());
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => emit_string_array_fill_aarch64(ctx, names),
        Arch::X86_64 => emit_string_array_fill_x86_64(ctx, names),
    }
    Ok(())
}

/// Appends method names to the current result array on AArch64.
fn emit_string_array_fill_aarch64(ctx: &mut FunctionContext<'_>, names: &[String]) {
    ctx.emitter.instruction("str x0, [sp, #-16]!");                             // park the method-name array while appending names
    for name in names {
        let (label, len) = ctx.data.add_string(name.as_bytes());
        ctx.emitter.instruction("ldr x0, [sp]");                                // reload the method-name array for this append
        abi::emit_symbol_address(ctx.emitter, "x1", &label);
        abi::emit_load_int_immediate(ctx.emitter, "x2", len as i64);
        abi::emit_call_label(ctx.emitter, "__rt_array_push_str");
        ctx.emitter.instruction("str x0, [sp]");                                // preserve the possibly-grown method-name array
    }
    ctx.emitter.instruction("ldr x0, [sp], #16");                               // restore the final method-name array as the result
}

/// Appends method names to the current result array on x86_64.
fn emit_string_array_fill_x86_64(ctx: &mut FunctionContext<'_>, names: &[String]) {
    ctx.emitter.instruction("push rax");                                        // park the method-name array while appending names
    ctx.emitter.instruction("sub rsp, 8");                                      // keep stack alignment stable across append helper calls
    for name in names {
        let (label, len) = ctx.data.add_string(name.as_bytes());
        ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 8]");                // reload the method-name array for this append
        abi::emit_symbol_address(ctx.emitter, "rsi", &label);
        abi::emit_load_int_immediate(ctx.emitter, "rdx", len as i64);
        abi::emit_call_label(ctx.emitter, "__rt_array_push_str");
        ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rax");                // preserve the possibly-grown method-name array
    }
    ctx.emitter.instruction("add rsp, 8");                                      // drop the temporary alignment slot
    ctx.emitter.instruction("pop rax");                                         // restore the final method-name array as the result
}
