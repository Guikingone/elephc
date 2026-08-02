//! Purpose:
//! Lowers `get_class_methods(object|string): array` for the EIR backend:
//! calling-scope method-name visibility (public-only from outside a class,
//! public+protected+own-private from inside a matching method), in PHP
//! declaration order.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::lower_builtin_call()`.
//!
//! Key details:
//! - COMPILE-TIME BAKE, matching `ReflectionClass::getMethods()`'s own strategy
//!   (`crate::codegen::lower_inst::objects::reflection`): the target class
//!   AND the calling scope are both resolved to compile-time-known names
//!   before this ever emits assembly, so the result is a plain compile-time
//!   `Vec<String>` materialized as an indexed string array — no runtime table
//!   walk needed. Follows PHP's `decl_order` semantics (declaration-order
//!   collapsing: an override keeps the position of the level that
//!   (re)declares it); this file owns its OWN small duplicate of the ancestor
//!   walk because it needs a DIFFERENT exposure filter than
//!   `ReflectionClass::getMethods()`'s (calling-scope visibility, not
//!   reflection's "everything but ancestor-private").
//! - Two lowerings, one behavior. A literal class-name string and an object
//!   argument with a concrete STATIC `PhpType::Object(name)` type are baked at
//!   compile time. Everything else — a computed string, a `Mixed`/union value,
//!   or a name that is not a declared class — resolves through the
//!   `_class_methods_table` registry at runtime
//!   (`crate::codegen_support::runtime::data::class_methods_registry`), with
//!   calling-scope visibility still decided at compile time because the
//!   enclosing class always is. A target that names no class raises PHP's own
//!   `TypeError`, php -n 8.5 verified.
//! - Residual: the object path resolves the argument's DECLARED type, not its
//!   (potentially more derived) runtime type, so a `Base`-typed variable holding
//!   a `Leaf` instance reports `Base`'s methods — unlike
//!   `class_implements()`/`class_parents()`, which go through the runtime class
//!   id. Left as-is deliberately: routing it through the runtime path would
//!   change a shape that already compiles.
//! - "Calling scope" is the ENCLOSING class of the current EIR function
//!   (`FunctionContext::function.name`'s `"Class::method"` prefix, mirroring
//!   `crate::codegen::mod::current_function_class`) when it matches the
//!   target class exactly (case-insensitively). Any other enclosing class (a
//!   different class in the same hierarchy) falls back to the public-only
//!   filter — never over-accepted as if it were self scope (php -n verified:
//!   only the EXACT declaring/receiving class's own scope sees non-public
//!   members via `get_class_methods`, not siblings/ancestors in general).

use std::collections::HashSet;

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen::{CodegenIrError, Result};
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
    if let Some(target_class) = optional_static_target_class(ctx, arg)? {
        let caller_class = current_function_class(ctx);
        let self_scope = caller_class
            .map(|c| php_symbol_key(c) == php_symbol_key(&target_class))
            .unwrap_or(false);
        let names = get_class_methods_names(ctx, &target_class, self_scope);
        emit_string_array(ctx, &names)?;
        return store_if_result(ctx, inst);
    }
    lower_dynamic_get_class_methods(ctx, arg)?;
    store_if_result(ctx, inst)
}

/// Lowers the shapes whose target class is only known at runtime: a non-literal class-name
/// string, or a `Mixed`/union value carrying one.
///
/// Structure mirrors `class_relations::emit_mixed_class_relation_lookup` — unbox, dispatch on
/// the runtime tag, funnel every path through one name-in-`string_result_regs` resolver — with
/// two `get_class_methods`-specific differences:
///
/// - **Calling-scope visibility stays a COMPILE-TIME decision.** PHP exposes non-public members
///   only to the exact declaring class's own scope, and the enclosing class of the EIR function
///   being lowered is known here. So the runtime name is compared against that one class name,
///   and a match branches to the self-scope list baked inline; everything else reads the
///   public-only list from `_class_methods_table`. No per-class visibility data is emitted.
/// - **A miss THROWS.** `class_implements()` returns `false` for an unknown name, but
///   `get_class_methods()` raises `TypeError: get_class_methods(): Argument #1
///   ($object_or_class) must be an object or a valid class name, string given` (php -n 8.5
///   verified). Returning `false` here would be a silently-wrong value, not a degraded one.
fn lower_dynamic_get_class_methods(ctx: &mut FunctionContext<'_>, value: ValueId) -> Result<()> {
    let object_label = ctx.next_label("gcm_mixed_obj");
    let string_label = ctx.next_label("gcm_mixed_str");
    let done_label = ctx.next_label("gcm_mixed_done");

    if matches!(ctx.value_php_type(value)?, PhpType::Str) {
        // Already a plain string: load it straight into the resolver's register pair.
        ctx.load_value_to_result(value)?;
        emit_dynamic_lookup_from_string_result(ctx, &done_label)?;
        ctx.emitter.label(&done_label);
        return Ok(());
    }

    ctx.load_value_to_result(value)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox"); // x0/rax=tag, x1/rdi=lo, x2/rdx=hi
    super::emit_branch_on_gettype_mixed_tag(ctx, 6, &object_label); // tag 6 = object
    super::emit_branch_on_gettype_mixed_tag(ctx, 1, &string_label); // tag 1 = string

    // -- neither an object nor a string payload: PHP's TypeError, same as an unknown name --
    emit_invalid_target_type_error(ctx);

    ctx.emitter.label(&object_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1"); // unboxed object pointer -> class-name lookup input
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rax, rdi"); // unboxed object pointer -> class-name lookup input
        }
    }
    super::types::emit_dynamic_object_class_name(ctx, "get_class_methods");
    emit_dynamic_lookup_from_string_result(ctx, &done_label)?;

    ctx.emitter.label(&string_label);
    if ctx.emitter.target.arch == Arch::X86_64 {
        ctx.emitter.instruction("mov rax, rdi"); // unboxed string pointer -> string_result_regs' ptr slot
    }
    // AArch64's unboxed (ptr, len) already sit in (x1, x2), the exact `abi::string_result_regs`
    // pair for this target, so no move is needed.
    emit_dynamic_lookup_from_string_result(ctx, &done_label)?;

    ctx.emitter.label(&done_label);
    Ok(())
}

/// Resolves the class name currently held in `abi::string_result_regs` and leaves the finished
/// method-name array as the current result, jumping to `done_label` on every success path.
///
/// Emits, in order: a case fold, the compile-time self-scope shortcut (when the enclosing class
/// is known), the `_class_methods_table` binary search, and the `TypeError` miss path.
///
/// The fold is what makes the search correct: `_class_methods_table` rows are stored
/// pre-lowercased (PHP class names are case-insensitive), so the query must be lowered first —
/// exactly what `__rt_class_exists` does before searching `_class_table`. `__rt_strtolower`
/// reads and returns its pair in the SAME registers `abi::string_result_regs` names, so the
/// call needs no shuffling on either target.
fn emit_dynamic_lookup_from_string_result(
    ctx: &mut FunctionContext<'_>,
    done_label: &str,
) -> Result<()> {
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    let miss_label = ctx.next_label("gcm_miss");
    let table_label = ctx.next_label("gcm_table");

    abi::emit_call_label(ctx.emitter, "__rt_strtolower"); // fold in place: PHP class names are case-insensitive

    // -- compile-time self-scope shortcut: does the runtime name name THIS class? --
    if let Some(self_names) = self_scope_names_worth_a_runtime_check(ctx) {
        let folded = php_symbol_key(&self_names.0);
        let (label, len) = ctx.data.add_string(folded.as_bytes());
        abi::emit_push_reg_pair(ctx.emitter, ptr_reg, len_reg); // park the folded name across the compare
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                // `__rt_strcmp` reads ptr_a/len_a in x1/x2 — already the folded name's registers.
                abi::emit_symbol_address(ctx.emitter, "x3", &label);
                abi::emit_load_int_immediate(ctx.emitter, "x4", len as i64);
                abi::emit_call_label(ctx.emitter, "__rt_strcmp"); // x0 = 0 when the names are equal
                ctx.emitter.instruction("cmp x0, #0");
                ctx.emitter.instruction(&format!("b.ne {}", table_label)); // a different class: use the table
            }
            Arch::X86_64 => {
                ctx.emitter.instruction(&format!("mov rdi, {}", ptr_reg)); // folded ptr -> strcmp ptr_a
                ctx.emitter.instruction(&format!("mov rsi, {}", len_reg)); // folded len -> strcmp len_a
                abi::emit_symbol_address(ctx.emitter, "rdx", &label);
                abi::emit_load_int_immediate(ctx.emitter, "rcx", len as i64);
                abi::emit_call_label(ctx.emitter, "__rt_strcmp"); // rax = 0 when the names are equal
                ctx.emitter.instruction("test rax, rax");
                ctx.emitter.instruction(&format!("jnz {}", table_label)); // a different class: use the table
            }
        }
        abi::emit_pop_reg_pair(ctx.emitter, ptr_reg, len_reg); // drop the parked name; this path needs no lookup
        emit_string_array(ctx, &self_names.1)?;
        abi::emit_jump(ctx.emitter, done_label);

        ctx.emitter.label(&table_label);
        abi::emit_pop_reg_pair(ctx.emitter, ptr_reg, len_reg); // restore the folded name for the search
    } else {
        ctx.emitter.label(&table_label);
    }

    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("mov x0, {}", ptr_reg)); // folded name_ptr -> search arg0
            ctx.emitter.instruction(&format!("mov x1, {}", len_reg)); // folded name_len -> search arg1
            abi::emit_symbol_address(ctx.emitter, "x2", "_class_methods_table");
            abi::emit_load_symbol_to_reg(ctx.emitter, "x3", "_class_methods_table_count", 0);
            abi::emit_load_int_immediate(ctx.emitter, "x4", ROW_SIZE);
            abi::emit_call_label(ctx.emitter, "__rt_sorted_name_search"); // x0 = matching row, or 0
            ctx.emitter.instruction(&format!("cbz x0, {}", miss_label)); // no class row carries this name
            ctx.emitter.instruction("ldr x1, [x0, #16]"); // row.methods_ptr
            ctx.emitter.instruction("ldr x2, [x0, #24]"); // row.methods_count
            ctx.emitter.instruction("mov x0, x1"); // list_ptr -> array builder arg0
            ctx.emitter.instruction("mov x1, x2"); // list_count -> array builder arg1
            abi::emit_call_label(ctx.emitter, "__rt_array_from_name_list"); // x0 = the built indexed array
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("mov rdi, {}", ptr_reg)); // folded name_ptr -> search arg0
            ctx.emitter.instruction(&format!("mov rsi, {}", len_reg)); // folded name_len -> search arg1
            abi::emit_symbol_address(ctx.emitter, "rdx", "_class_methods_table");
            abi::emit_load_symbol_to_reg(ctx.emitter, "rcx", "_class_methods_table_count", 0);
            abi::emit_load_int_immediate(ctx.emitter, "r8", ROW_SIZE);
            abi::emit_call_label(ctx.emitter, "__rt_sorted_name_search"); // rax = matching row, or 0
            ctx.emitter.instruction("test rax, rax");
            ctx.emitter.instruction(&format!("jz {}", miss_label)); // no class row carries this name
            ctx.emitter.instruction("mov rdi, QWORD PTR [rax + 16]"); // row.methods_ptr
            ctx.emitter.instruction("mov rsi, QWORD PTR [rax + 24]"); // row.methods_count
            abi::emit_call_label(ctx.emitter, "__rt_array_from_name_list"); // rax = the built indexed array
        }
    }
    abi::emit_jump(ctx.emitter, done_label);

    ctx.emitter.label(&miss_label);
    emit_invalid_target_type_error(ctx);
    Ok(())
}

/// Returns `(enclosing class name, its self-scope method list)` when the body being lowered sits
/// inside a class whose self-scope list actually DIFFERS from its public one — the only case
/// where a runtime name comparison can change the answer.
///
/// A class with no protected/private methods needs no check at all: the table's public list is
/// already what PHP reports from inside it too, so no comparison is emitted.
fn self_scope_names_worth_a_runtime_check(
    ctx: &FunctionContext<'_>,
) -> Option<(String, Vec<String>)> {
    let enclosing = current_function_class(ctx)?;
    let resolved = lookup_class_name(ctx, enclosing)?;
    let self_names = get_class_methods_names(ctx, &resolved, true);
    (self_names != get_class_methods_names(ctx, &resolved, false))
        .then_some((resolved, self_names))
}

/// Emits PHP 8.5's exact `get_class_methods()` rejection for a target that is neither an object
/// nor the name of a declared class (php -n verified, including the trailing "string given").
fn emit_invalid_target_type_error(ctx: &mut FunctionContext<'_>) {
    emit_throw_static_type_error(
        ctx,
        "get_class_methods(): Argument #1 ($object_or_class) must be an object or a valid \
         class name, string given",
    );
}

/// Constructs and throws a `\TypeError` carrying a compile-time-constant message.
///
/// A verbatim sibling of `builtins::arrays::unshift::emit_throw_static_type_error` (private to
/// that module), duplicated here per this directory's established precedent for small
/// single-use helpers — the same reason `lookup_class_name` is duplicated above.
fn emit_throw_static_type_error(ctx: &mut FunctionContext<'_>, message: &str) {
    let (message_label, message_len) = ctx.data.add_string(message.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x1", &message_label); // message pointer
            ctx.emitter.instruction(&format!("mov x2, #{}", message_len)); // message byte length
            abi::emit_call_label(ctx.emitter, "__rt_str_persist"); // own a heap copy of the message bytes
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2"); // park the owned message across the allocation call
            ctx.emitter.instruction("mov x0, #32"); // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc"); // allocate the TypeError object payload
            ctx.emitter.instruction("mov x9, #6"); // heap kind 6 = object instance
            ctx.emitter.instruction("str x9, [x0, #-8]"); // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "x9", "_spl_type_error_class_id", 0);
            ctx.emitter.instruction("str x9, [x0]"); // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "x9", "x10"); // reload the owned message pointer/length
            ctx.emitter.instruction("str x9, [x0, #8]"); // store the exception message pointer
            ctx.emitter.instruction("str x10, [x0, #16]"); // store the exception message length
            ctx.emitter.instruction("str xzr, [x0, #24]"); // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "x0", "_exc_value", 0); // publish the active exception
            abi::emit_jump(ctx.emitter, "__rt_throw_current"); // enter the standard exception unwinder
        }
        Arch::X86_64 => {
            // `__rt_str_persist`'s real x86_64 input convention is rax=ptr, rdx=len (see the note
            // in `builtins::arrays::unshift`: its own header comment is stale).
            abi::emit_symbol_address(ctx.emitter, "rax", &message_label); // message pointer
            ctx.emitter.instruction(&format!("mov rdx, {}", message_len)); // message byte length
            abi::emit_call_label(ctx.emitter, "__rt_str_persist"); // own a heap copy of the message
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx"); // park the owned message across the allocation call
            ctx.emitter.instruction("mov rax, 32"); // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc"); // allocate the TypeError object payload
            ctx.emitter.instruction("mov r10, 0x4548504c00000006"); // x86_64 heap-kind word: object magic + kind 6
            ctx.emitter.instruction("mov QWORD PTR [rax - 8], r10"); // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "r10", "_spl_type_error_class_id", 0);
            ctx.emitter.instruction("mov QWORD PTR [rax], r10"); // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "r10", "r11"); // reload the owned message pointer/length
            ctx.emitter.instruction("mov QWORD PTR [rax + 8], r10"); // store the exception message pointer
            ctx.emitter.instruction("mov QWORD PTR [rax + 16], r11"); // store the exception message length
            ctx.emitter.instruction("mov QWORD PTR [rax + 24], 0"); // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "rax", "_exc_value", 0); // publish the active exception
            abi::emit_jump(ctx.emitter, "__rt_throw_current"); // enter the standard exception unwinder
        }
    }
}

/// Byte size of one `_class_methods_table` row (`{name_ptr, name_len, list_ptr, list_count}`).
/// Deliberately NARROWER than the relation tables' shared 64-byte row — see
/// `crate::codegen_support::runtime::data::class_methods_registry`.
const ROW_SIZE: i64 = 32;

/// Returns the compile-time-known target class, or `None` when the answer must come from the
/// runtime path.
///
/// `None` covers two different situations that the dynamic lowering handles identically and
/// correctly: a target only known at runtime, AND a literal/statically-typed name that is not a
/// declared class. The second used to be a `CodegenIrError::unsupported` — a compile error for
/// something PHP compiles and then rejects with a CATCHABLE `TypeError` at the call. Routing it
/// through the runtime path costs one binary search that is known to miss, and buys the exact
/// PHP behavior with a single copy of the message.
///
/// The object case stays a compile-time bake against the DECLARED type (keeping the documented
/// residual that a `Base`-typed variable holding a `Leaf` reports `Base`'s methods, unlike
/// `class_implements()`), so adding runtime resolution changes NO shape that already compiled.
fn optional_static_target_class(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
) -> Result<Option<String>> {
    if let Some(literal) = optional_const_string_operand(ctx, value)? {
        return Ok(lookup_class_name(ctx, literal.trim_start_matches('\\')));
    }
    match ctx.value_php_type(value)? {
        PhpType::Object(name) => Ok(lookup_class_name(ctx, &name)),
        _ => Ok(None),
    }
}

/// Returns the class encoded in the CURRENT EIR function's `"Class::method"`
/// name, or `None` for a free function/top-level scope. Mirrors
/// `crate::codegen::mod::current_function_class` (private to that module).
fn current_function_class<'a>(ctx: &'a FunctionContext<'_>) -> Option<&'a str> {
    ctx.function.name.rsplit_once("::").map(|(class_name, _)| class_name)
}

/// Looks up a class by PHP-style case-insensitive name (mirrors
/// `crate::codegen::lower_inst::builtins::class_relations::lookup_class_name`,
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
/// (re)declares it — matches PHP's `ReflectionClass::getMethods()` decl_order
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
