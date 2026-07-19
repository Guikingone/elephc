//! Purpose:
//! Lowers dynamic `new ReflectionMethod($class, $method)` / `new ReflectionProperty($class,
//! $property)` construction (both arguments runtime strings) through the flat, class-id-indexed
//! method/property registry tables (`crate::codegen::runtime::data::reflect_member_registry`).
//! Two shared, program-wide dispatcher labels are emitted (once, only if actually used) —
//! `_elephc_reflect_method_new_dynamic` / `_elephc_reflect_property_new_dynamic` — mirroring
//! `crate::codegen_ir::lower_inst::objects::reflection`'s existing dynamic `ReflectionClass(name)`
//! dispatcher, but TABLE-DRIVEN (a single binary search per lookup) instead of a per-class linear
//! compare chain: the linear-chain shape does not scale to per-(class, member) granularity (recon
//! measured it 6x over the emitted-code-size budget for this feature).
//!
//! Called from:
//! - `crate::codegen_ir::lower_inst::objects::reflection::lower_reflection_owner_new()` routes a
//!   non-literal `ReflectionMethod`/`ReflectionProperty` constructor operand here instead of the
//!   compile-time metadata bake.
//! - `crate::codegen_ir::generate_user_asm_from_ir_with_options()` calls
//!   `emit_reflection_member_dynamic_dispatch_if_needed()` once, after per-function lowering.
//! - `crate::codegen_ir::finalize_user_asm()` emits the backing data tables (gated by the same
//!   `module_needs_reflection_member_dynamic_dispatch()` scan).
//!
//! Key details:
//! - SCOPE: this delivery accepts a `Str`-typed class-name AND member-name argument only (not
//!   `object|string` weak coercion like `ReflectionClass`'s dynamic path) — a documented, narrower
//!   scope; `crate::types::checker::inference::objects::constructors::reflection_class_literal_arg`
//!   rejects any other type at compile time with a loud diagnostic, never a crash.
//! - `getMethods()`/`getProperties()` array enumeration is NOT implemented by this delivery (the
//!   table's `declaring_class_id` field exists specifically so a future enumeration pass does not
//!   need a table format change) — see the `reflect_member_registry` module doc comment.
//! - Every dispatcher uses ONE fixed-size reserved temporary stack frame (not LIFO push/pop
//!   juggling) so field offsets stay constant across the intervening runtime calls
//!   (`__rt_strtolower`/`__rt_sorted_name_search`/`__rt_str_persist` all clobber caller-saved
//!   registers); values that must survive a call are always spilled to a NAMED frame offset first.
//! - `getAttributes()` on a dynamically-constructed `ReflectionMethod`/`ReflectionProperty` returns
//!   an empty array (the `__attrs` shell default) — a documented, honest limitation: the flat table
//!   does not carry per-row attribute metadata (that would need yet another payload table); this is
//!   loud-enough (an always-empty array, never a fabricated non-empty one) rather than silently
//!   wrong.

use crate::codegen::abi;
use crate::codegen::data_section::DataSection;
use crate::codegen::emit::Emitter;
use crate::codegen::platform::Arch;
use crate::codegen::runtime::{
    CLASS_ID_ROW_CLASS_ID_OFFSET, CLASS_ID_ROW_SIZE, INDEX_ROW_SIZE, METHOD_ROW_MODIFIERS_OFFSET,
    METHOD_ROW_REAL_NAME_LEN_OFFSET, METHOD_ROW_REAL_NAME_PTR_OFFSET, METHOD_ROW_SIZE,
    PROPERTY_ROW_MODIFIERS_OFFSET, PROPERTY_ROW_SIZE,
};
use crate::codegen_ir::{CodegenIrError, Result};
use crate::ir::{Function, Immediate, Instruction, IrType, Module, Op, ValueId};
use crate::types::PhpType;

use super::super::super::context::FunctionContext;
use super::super::super::frame;
use super::reflection::{emit_reflection_attrs_property, is_const_string_or_class_value};

/// Assembly label of the shared, program-wide dynamic `ReflectionMethod(class, method)`
/// construction dispatcher.
const METHOD_DISPATCH_LABEL: &str = "_elephc_reflect_method_new_dynamic";
/// Assembly label of the shared, program-wide dynamic `ReflectionProperty(class, property)`
/// construction dispatcher.
const PROPERTY_DISPATCH_LABEL: &str = "_elephc_reflect_property_new_dynamic";

/// Returns true when `inst` is an `Op::ObjectNew` constructing `class_name` with at least one
/// non-literal `Str` operand among its first two operands.
fn object_new_is_dynamic_member_ctor(function: &Function, inst: &Instruction, class_name: &str) -> bool {
    let Some(Immediate::Data(data)) = inst.immediate else {
        return false;
    };
    // (class-name lookup happens in the caller via `object_new_class_name`; here we only need
    // to know whether ANY of the first two operands is non-literal.)
    let _ = class_name;
    let _ = data;
    inst.operands
        .iter()
        .take(2)
        .any(|&value| !is_const_string_or_class_value(function, value))
}

/// Iterates every function-like body lowered into the EIR module. Duplicated in miniature from
/// `super::reflection::reflection_dispatch_scan_functions` (private there); this file lives in a
/// sibling module and the scan is a one-line chain, not worth widening that function's visibility
/// for.
fn scan_functions(module: &Module) -> impl Iterator<Item = &Function> {
    module
        .functions
        .iter()
        .chain(module.class_methods.iter())
        .chain(module.closures.iter())
        .chain(module.fiber_wrappers.iter())
        .chain(module.callback_wrappers.iter())
        .chain(module.extern_callback_trampolines.iter())
        .chain(module.runtime_callable_invokers.iter())
}

/// Returns the class name an `Op::ObjectNew` instruction constructs, or `None` for a malformed
/// instruction.
fn object_new_class_name<'a>(module: &'a Module, inst: &Instruction) -> Option<&'a str> {
    let Some(Immediate::Data(data)) = inst.immediate else {
        return None;
    };
    module.data.class_names.get(data.as_raw() as usize).map(String::as_str)
}

/// Returns true when the module contains at least one dynamic-argument `new
/// ReflectionMethod(...)`/`new ReflectionProperty(...)` construction site — i.e. whether the
/// shared dispatchers and their backing data tables need to be emitted at all.
pub(crate) fn module_needs_reflection_member_dynamic_dispatch(module: &Module) -> bool {
    scan_functions(module).any(|function| {
        function.instructions.iter().any(|inst| {
            inst.op == Op::ObjectNew
                && matches!(object_new_class_name(module, inst), Some("ReflectionMethod") | Some("ReflectionProperty"))
                && object_new_is_dynamic_member_ctor(
                    function,
                    inst,
                    object_new_class_name(module, inst).unwrap_or(""),
                )
        })
    })
}

/// Emits both shared dynamic dispatchers (`_elephc_reflect_method_new_dynamic` /
/// `_elephc_reflect_property_new_dynamic`), once, if and only if the module actually needs them
/// (see `module_needs_reflection_member_dynamic_dispatch`). No-op otherwise.
pub(crate) fn emit_reflection_member_dynamic_dispatch_if_needed(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
) -> Result<()> {
    if !module_needs_reflection_member_dynamic_dispatch(module) {
        return Ok(());
    }
    let target = emitter.target;
    let synthetic = Function::new(format!("{}_impl", METHOD_DISPATCH_LABEL), IrType::Void, PhpType::Void);
    let layout = frame::layout_for_function(&synthetic, target, false);
    let mut ctx = FunctionContext::new(module, &synthetic, emitter, data, layout, false, false, false, None);
    emit_method_dispatcher(&mut ctx)?;
    emit_property_dispatcher(&mut ctx)?;
    Ok(())
}

/// Lowers `new ReflectionMethod($class, $method)` for a dynamic-argument call site: materializes
/// the class-name and method-name operands into the dispatcher's argument convention and calls
/// `METHOD_DISPATCH_LABEL`. Both operands must be statically `Str`-typed — the checker
/// (`reflection_class_literal_arg`/`reflection_member_name_arg`) only routes here for that shape.
pub(super) fn lower_reflection_method_new_dynamic(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    class_operand: ValueId,
    method_operand: ValueId,
) -> Result<()> {
    lower_reflection_member_new_dynamic(ctx, inst, class_operand, method_operand, METHOD_DISPATCH_LABEL, "ReflectionMethod")
}

/// Lowers `new ReflectionProperty($class, $property)` for a dynamic-argument call site. See
/// `lower_reflection_method_new_dynamic`.
pub(super) fn lower_reflection_property_new_dynamic(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    class_operand: ValueId,
    property_operand: ValueId,
) -> Result<()> {
    lower_reflection_member_new_dynamic(
        ctx,
        inst,
        class_operand,
        property_operand,
        PROPERTY_DISPATCH_LABEL,
        "ReflectionProperty",
    )
}

/// Shared operand-materialization + dispatch-call lowering for both
/// `lower_reflection_method_new_dynamic`/`lower_reflection_property_new_dynamic`. Both operands
/// must resolve to `Str` at codegen time (checker-enforced); anything else is an internal error
/// (never reachable for a module that passed type checking).
fn lower_reflection_member_new_dynamic(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    class_operand: ValueId,
    member_operand: ValueId,
    dispatch_label: &str,
    owner: &str,
) -> Result<()> {
    if !matches!(ctx.value_php_type(class_operand)?, PhpType::Str) {
        return Err(CodegenIrError::unsupported(format!(
            "{} dynamic constructor with a non-string class-name argument",
            owner
        )));
    }
    if !matches!(ctx.value_php_type(member_operand)?, PhpType::Str) {
        return Err(CodegenIrError::unsupported(format!(
            "{} dynamic constructor with a non-string member-name argument",
            owner
        )));
    }
    // Materialize the member-name operand FIRST and park it, since materializing the class-name
    // operand next would otherwise overwrite the same string-result registers.
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.load_string_value_to_regs(member_operand, "x0", "x1")?,
        Arch::X86_64 => ctx.load_string_value_to_regs(member_operand, "rdi", "rsi")?,
    }
    abi::emit_push_reg_pair(
        ctx.emitter,
        match ctx.emitter.target.arch {
            Arch::AArch64 => "x0",
            Arch::X86_64 => "rdi",
        },
        match ctx.emitter.target.arch {
            Arch::AArch64 => "x1",
            Arch::X86_64 => "rsi",
        },
    );
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.load_string_value_to_regs(class_operand, "x0", "x1")?,
        Arch::X86_64 => ctx.load_string_value_to_regs(class_operand, "rdi", "rsi")?,
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => abi::emit_pop_reg_pair(ctx.emitter, "x2", "x3"),
        Arch::X86_64 => abi::emit_pop_reg_pair(ctx.emitter, "rdx", "rcx"),
    }
    abi::emit_call_label(ctx.emitter, dispatch_label);
    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("reflection object_new missing result"))?;
    ctx.store_result_value(result)
}

/// Frame offsets for the method dispatcher's fixed 96-byte reserved temporary frame.
mod method_frame {
    pub(super) const SIZE: usize = 96;
    pub(super) const CLASS_PTR: usize = 0;
    pub(super) const CLASS_LEN: usize = 8;
    pub(super) const METHOD_PTR: usize = 16;
    pub(super) const METHOD_LEN: usize = 24;
    pub(super) const CLASS_ID: usize = 32;
    pub(super) const LC_METHOD_PTR: usize = 40;
    pub(super) const LC_METHOD_LEN: usize = 48;
    pub(super) const REAL_NAME_PTR: usize = 56;
    pub(super) const REAL_NAME_LEN: usize = 64;
    pub(super) const MODIFIERS: usize = 72;
}

/// Frame offsets for the property dispatcher's fixed 64-byte reserved temporary frame.
mod property_frame {
    pub(super) const SIZE: usize = 64;
    pub(super) const CLASS_PTR: usize = 0;
    pub(super) const CLASS_LEN: usize = 8;
    pub(super) const PROPERTY_PTR: usize = 16;
    pub(super) const PROPERTY_LEN: usize = 24;
    pub(super) const CLASS_ID: usize = 32;
    pub(super) const MODIFIERS: usize = 40;
}

/// Emits the standard leaf-function prologue: save frame pointer/return address, establish the
/// new frame pointer. Matches `crate::codegen::runtime::system::rt_class_exists`'s shape.
fn emit_prologue(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("stp x29, x30, [sp, #-16]!");               // save frame pointer and return address
            ctx.emitter.instruction("mov x29, sp");                             // establish the new frame pointer
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("push rbp");                                // preserve the caller frame pointer
            ctx.emitter.instruction("mov rbp, rsp");                            // establish an aligned helper frame
        }
    }
}

/// Emits the matching epilogue for a SUCCESS return: the constructed object pointer is already in
/// the ABI integer result register.
fn emit_epilogue(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("ldp x29, x30, [sp], #16");                 // restore frame pointer and return address
            ctx.emitter.instruction("ret");                                     // return the constructed object pointer in x0
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("pop rbp");                                 // restore the caller frame pointer
            ctx.emitter.instruction("ret");                                     // return the constructed object pointer in rax
        }
    }
}

/// Resolves the class-id of the class name stored at `class_ptr_off`/`class_len_off` in the
/// current reserved frame: lowercases the query (stripping one leading `\`), binary-searches
/// `_reflect_class_id_table`, and on a miss jumps to `not_found_label` (never returns from there).
/// On a hit, stores the resolved `class_id` at `class_id_off` in the frame and falls through.
fn emit_resolve_class_id(
    ctx: &mut FunctionContext<'_>,
    class_ptr_off: usize,
    class_len_off: usize,
    class_id_off: usize,
    not_found_label: &str,
) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x0", class_ptr_off); // reload the original class-name pointer
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", class_len_off); // reload the original class-name length
            emit_strip_leading_backslash(ctx, "x0", "x1", "x9");
            ctx.emitter.instruction("mov x2, x1");                              // __rt_strtolower expects the length in x2
            ctx.emitter.instruction("mov x1, x0");                              // __rt_strtolower expects the pointer in x1
            abi::emit_call_label(ctx.emitter, "__rt_strtolower");               // x1=lowercased class-name copy, x2=length
            ctx.emitter.instruction("mov x0, x1");                              // lowercased pointer -> search arg0
            ctx.emitter.instruction("mov x1, x2");                              // lowercased length -> search arg1
            abi::emit_symbol_address(ctx.emitter, "x2", "_reflect_class_id_table"); // table base -> search arg2
            abi::emit_load_symbol_to_reg(ctx.emitter, "x3", "_reflect_class_id_table_count", 0); // entry count -> search arg3
            ctx.emitter.instruction(&format!("mov x4, #{}", CLASS_ID_ROW_SIZE)); // class-id-table entries are CLASS_ID_ROW_SIZE bytes wide
            abi::emit_call_label(ctx.emitter, "__rt_sorted_name_search");       // search for the class name -> x0 = entry ptr or 0
            ctx.emitter.instruction(&format!("cbz x0, {}", not_found_label));   // no matching class -> throw "class not found"
            ctx.emitter.instruction(&format!("ldr x5, [x0, #{}]", CLASS_ID_ROW_CLASS_ID_OFFSET)); // load the resolved class_id from the matched entry
            abi::emit_store_to_sp(ctx.emitter, "x5", class_id_off);             // spill class_id across the calls that follow
        }
        Arch::X86_64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", class_ptr_off); // reload the original class-name pointer
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rsi", class_len_off); // reload the original class-name length
            emit_strip_leading_backslash(ctx, "rdi", "rsi", "r9");
            ctx.emitter.instruction("mov rax, rdi");                            // __rt_strtolower expects the pointer in rax
            ctx.emitter.instruction("mov rdx, rsi");                            // and the length in rdx
            abi::emit_call_label(ctx.emitter, "__rt_strtolower");               // rax=lowercased class-name copy, rdx=length
            ctx.emitter.instruction("mov rdi, rax");                            // lowercased pointer -> search arg0
            ctx.emitter.instruction("mov rsi, rdx");                            // lowercased length -> search arg1
            abi::emit_symbol_address(ctx.emitter, "rdx", "_reflect_class_id_table"); // table base -> search arg2
            abi::emit_load_symbol_to_reg(ctx.emitter, "rcx", "_reflect_class_id_table_count", 0); // entry count -> search arg3
            ctx.emitter.instruction(&format!("mov r8, {}", CLASS_ID_ROW_SIZE)); // class-id-table entries are CLASS_ID_ROW_SIZE bytes wide
            abi::emit_call_label(ctx.emitter, "__rt_sorted_name_search");       // search for the class name -> rax = entry ptr or 0
            ctx.emitter.instruction("test rax, rax");                           // did the search return a matching entry?
            ctx.emitter.instruction(&format!("jz {}", not_found_label));        // no matching class -> throw "class not found"
            ctx.emitter.instruction(&format!("mov r9, QWORD PTR [rax + {}]", CLASS_ID_ROW_CLASS_ID_OFFSET)); // load the resolved class_id from the matched entry
            abi::emit_store_to_sp(ctx.emitter, "r9", class_id_off);            // spill class_id across the calls that follow
        }
    }
}

/// Strips exactly one leading `\` from a (ptr, len) pair held in `ptr_reg`/`len_reg`, using
/// `scratch_reg` to peek the first byte. Mirrors
/// `super::reflection::emit_dynamic_dispatch_query_normalization`'s backslash-strip half.
fn emit_strip_leading_backslash(ctx: &mut FunctionContext<'_>, ptr_reg: &str, len_reg: &str, scratch_reg: &str) {
    let skip_label = ctx.next_label("reflect_member_dyn_skip_bs");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter
                .instruction(&format!("cbz {}, {}", len_reg, skip_label));      // an empty query cannot start with a leading backslash
            ctx.emitter
                .instruction(&format!("ldrb w{}, [{}]", scratch_reg.trim_start_matches('x'), ptr_reg)); // peek at the query's first byte
            ctx.emitter
                .instruction(&format!("cmp w{}, #0x5c", scratch_reg.trim_start_matches('x'))); // is it a leading namespace-root backslash?
            ctx.emitter.instruction(&format!("b.ne {}", skip_label));           // no backslash to strip
            ctx.emitter.instruction(&format!("add {}, {}, #1", ptr_reg, ptr_reg)); // strip the leading backslash from the working pointer
            ctx.emitter.instruction(&format!("sub {}, {}, #1", len_reg, len_reg)); // and from the working length
            ctx.emitter.label(&skip_label);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("test {}, {}", len_reg, len_reg)); // an empty query cannot start with a leading backslash
            ctx.emitter.instruction(&format!("jz {}", skip_label));
            ctx.emitter
                .instruction(&format!("movzx {}d, BYTE PTR [{}]", scratch_reg, ptr_reg)); // peek at the query's first byte
            ctx.emitter
                .instruction(&format!("cmp {}b, 0x5c", scratch_reg));           // is it a leading namespace-root backslash?
            ctx.emitter.instruction(&format!("jne {}", skip_label));            // no backslash to strip
            ctx.emitter.instruction(&format!("add {}, 1", ptr_reg));            // strip the leading backslash from the working pointer
            ctx.emitter.instruction(&format!("sub {}, 1", len_reg));            // and from the working length
            ctx.emitter.label(&skip_label);
        }
    }
}

/// Throws a catchable `\ReflectionException` echoing the ORIGINAL (unmodified) query name(s),
/// reloaded from fixed frame offsets. Shared shape for the class-not-found and method/property-
/// not-found throws below (mirrors `super::reflection::emit_reflection_class_not_found_throw`'s
/// allocate/stamp/publish/jump sequence). Builds the message as
/// `prefix + [first query] + infix + [second query, if given] + suffix` — e.g. `("Method ",
/// class, "::", Some(method), "() does not exist")` for the method-miss message, or `("Class \"",
/// class, "", None, "\" does not exist")` for the class-miss message (empty infix, no second
/// query).
fn emit_reflect_exception_throw(
    ctx: &mut FunctionContext<'_>,
    prefix: &[u8],
    first_ptr_off: usize,
    first_len_off: usize,
    infix: &[u8],
    second: Option<(usize, usize)>,
    suffix: &[u8],
) -> Result<()> {
    let (prefix_label, prefix_len) = ctx.data.add_string(prefix);
    let (infix_label, infix_len) = ctx.data.add_string(infix);
    let (suffix_label, suffix_len) = ctx.data.add_string(suffix);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x1", &prefix_label);         // message prefix pointer
            ctx.emitter.instruction(&format!("mov x2, #{}", prefix_len));       // message prefix byte length
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x3", first_ptr_off); // original first query pointer
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x4", first_len_off); // original first query byte length
            abi::emit_call_label(ctx.emitter, "__rt_concat");                   // prefix concatenated with the first original query
            if !infix.is_empty() || second.is_some() {
                ctx.emitter.instruction("mov x9, x1");                          // stash the running message pointer across the infix concat
                ctx.emitter.instruction("mov x10, x2");                         // stash the running message length
                abi::emit_symbol_address(ctx.emitter, "x3", &infix_label);      // infix text (e.g. "::") pointer
                ctx.emitter.instruction(&format!("mov x4, #{}", infix_len));    // infix text byte length
                ctx.emitter.instruction("mov x1, x9");                          // restore the running message pointer for concat's arg0
                ctx.emitter.instruction("mov x2, x10");                         // restore the running message length for concat's arg0
                abi::emit_call_label(ctx.emitter, "__rt_concat");               // append the infix text
            }
            if let Some((ptr_off, len_off)) = second {
                ctx.emitter.instruction("mov x9, x1");                          // stash the running message pointer across the second-query reload
                ctx.emitter.instruction("mov x10, x2");                         // stash the running message length
                abi::emit_load_temporary_stack_slot(ctx.emitter, "x3", ptr_off); // second original query pointer
                abi::emit_load_temporary_stack_slot(ctx.emitter, "x4", len_off); // second original query byte length
                ctx.emitter.instruction("mov x1, x9");                          // restore the running message pointer for concat's arg0
                ctx.emitter.instruction("mov x2, x10");                         // restore the running message length for concat's arg0
                abi::emit_call_label(ctx.emitter, "__rt_concat");               // append the second original query
            }
            abi::emit_symbol_address(ctx.emitter, "x3", &suffix_label);         // message suffix pointer
            ctx.emitter.instruction(&format!("mov x4, #{}", suffix_len));       // message suffix byte length
            abi::emit_call_label(ctx.emitter, "__rt_concat");                   // append the closing suffix
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");              // own a heap copy of the message bytes
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");                   // park the owned message across the allocation call
            ctx.emitter.instruction("mov x0, #32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");               // allocate the ReflectionException object payload
            ctx.emitter.instruction("mov x9, #6");                              // heap kind 6 = object instance
            ctx.emitter.instruction("str x9, [x0, #-8]");                       // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "x9", "_reflection_exception_class_id", 0); // load ReflectionException's runtime class id
            ctx.emitter.instruction("str x9, [x0]");                            // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "x9", "x10");                   // reload the owned message pointer/length
            ctx.emitter.instruction("str x9, [x0, #8]");                        // store the exception message pointer
            ctx.emitter.instruction("str x10, [x0, #16]");                      // store the exception message length
            ctx.emitter.instruction("str xzr, [x0, #24]");                      // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "x0", "_exc_value", 0);  // publish the active exception object
            abi::emit_jump(ctx.emitter, "__rt_throw_current");                  // enter the standard exception unwinder
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "rax", &prefix_label);        // message prefix pointer
            ctx.emitter.instruction(&format!("mov rdx, {}", prefix_len));       // message prefix byte length
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", first_ptr_off); // original first query pointer
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rsi", first_len_off); // original first query byte length
            abi::emit_call_label(ctx.emitter, "__rt_concat");                   // prefix concatenated with the first original query
            if !infix.is_empty() || second.is_some() {
                ctx.emitter.instruction("mov r9, rax");                         // stash the running message pointer across the infix concat
                ctx.emitter.instruction("mov r10, rdx");                        // stash the running message length
                abi::emit_symbol_address(ctx.emitter, "rdi", &infix_label);     // infix text (e.g. "::") pointer
                ctx.emitter.instruction(&format!("mov rsi, {}", infix_len));    // infix text byte length
                ctx.emitter.instruction("mov rax, r9");                         // restore the running message pointer for concat's arg0
                ctx.emitter.instruction("mov rdx, r10");                        // restore the running message length for concat's arg0
                abi::emit_call_label(ctx.emitter, "__rt_concat");               // append the infix text
            }
            if let Some((ptr_off, len_off)) = second {
                ctx.emitter.instruction("mov r9, rax");                         // stash the running message pointer across the second-query reload
                ctx.emitter.instruction("mov r10, rdx");                        // stash the running message length
                abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", ptr_off); // second original query pointer
                abi::emit_load_temporary_stack_slot(ctx.emitter, "rsi", len_off); // second original query byte length
                ctx.emitter.instruction("mov rax, r9");                         // restore the running message pointer for concat's arg0
                ctx.emitter.instruction("mov rdx, r10");                        // restore the running message length for concat's arg0
                abi::emit_call_label(ctx.emitter, "__rt_concat");               // append the second original query
            }
            abi::emit_symbol_address(ctx.emitter, "rdi", &suffix_label);        // message suffix pointer
            ctx.emitter.instruction(&format!("mov rsi, {}", suffix_len));       // message suffix byte length
            abi::emit_call_label(ctx.emitter, "__rt_concat");                   // append the closing suffix
            ctx.emitter.instruction("mov rdi, rax");                            // move the message pointer into the persist argument
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");              // own a heap copy of the message (ptr in rax, len carried in rdx)
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");                 // park the owned message across the allocation call
            ctx.emitter.instruction("mov rax, 32");                             // request Throwable payload storage
            abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");               // allocate the ReflectionException object payload
            ctx.emitter.instruction("mov r10, 0x4548504c00000006");             // x86_64 heap-kind word: object magic + kind 6
            ctx.emitter.instruction("mov QWORD PTR [rax - 8], r10");            // stamp the allocation as a runtime object
            abi::emit_load_symbol_to_reg(ctx.emitter, "r10", "_reflection_exception_class_id", 0); // load ReflectionException's runtime class id
            ctx.emitter.instruction("mov QWORD PTR [rax], r10");                // store the class id at the object header
            abi::emit_pop_reg_pair(ctx.emitter, "r10", "r11");                  // reload the owned message pointer/length
            ctx.emitter.instruction("mov QWORD PTR [rax + 8], r10");            // store the exception message pointer
            ctx.emitter.instruction("mov QWORD PTR [rax + 16], r11");           // store the exception message length
            ctx.emitter.instruction("mov QWORD PTR [rax + 24], 0");             // exception code defaults to zero
            abi::emit_store_reg_to_symbol(ctx.emitter, "rax", "_exc_value", 0); // publish the active exception object
            abi::emit_jump(ctx.emitter, "__rt_throw_current");                  // enter the standard exception unwinder
        }
    }
    Ok(())
}

/// Emits `_elephc_reflect_method_new_dynamic`: resolves `(class_ptr, class_len, method_ptr,
/// method_len)` to a `ReflectionMethod` shell object, or throws a catchable `\ReflectionException`
/// (php -n verified messages: `Class "NAME" does not exist"` / `Method CLASS::NAME() does not
/// exist`).
fn emit_method_dispatcher(ctx: &mut FunctionContext<'_>) -> Result<()> {
    use method_frame as f;
    ctx.emitter.blank();
    ctx.emitter
        .comment("--- reflection: dynamic ReflectionMethod(class, method) construction dispatch ---");
    ctx.emitter.label_global(METHOD_DISPATCH_LABEL);
    emit_prologue(ctx);
    abi::emit_reserve_temporary_stack(ctx.emitter, f::SIZE);

    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_store_to_sp(ctx.emitter, "x0", f::CLASS_PTR);             // spill the original class-name pointer
            abi::emit_store_to_sp(ctx.emitter, "x1", f::CLASS_LEN);             // spill the original class-name length
            abi::emit_store_to_sp(ctx.emitter, "x2", f::METHOD_PTR);            // spill the original method-name pointer
            abi::emit_store_to_sp(ctx.emitter, "x3", f::METHOD_LEN);            // spill the original method-name length
        }
        Arch::X86_64 => {
            abi::emit_store_to_sp(ctx.emitter, "rdi", f::CLASS_PTR);            // spill the original class-name pointer
            abi::emit_store_to_sp(ctx.emitter, "rsi", f::CLASS_LEN);            // spill the original class-name length
            abi::emit_store_to_sp(ctx.emitter, "rdx", f::METHOD_PTR);           // spill the original method-name pointer
            abi::emit_store_to_sp(ctx.emitter, "rcx", f::METHOD_LEN);           // spill the original method-name length
        }
    }

    let class_not_found = format!("{}_class_not_found", METHOD_DISPATCH_LABEL);
    let method_not_found = format!("{}_method_not_found", METHOD_DISPATCH_LABEL);
    emit_resolve_class_id(ctx, f::CLASS_PTR, f::CLASS_LEN, f::CLASS_ID, &class_not_found);

    // -- lowercase the method name (methods are case-insensitive, never namespaced) --
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", f::METHOD_PTR); // reload the original method-name pointer
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x2", f::METHOD_LEN); // reload the original method-name length
            abi::emit_call_label(ctx.emitter, "__rt_strtolower");               // x1=lowercased method-name copy, x2=length
            abi::emit_store_to_sp(ctx.emitter, "x1", f::LC_METHOD_PTR);         // spill the lowercased pointer across the segment lookup below
            abi::emit_store_to_sp(ctx.emitter, "x2", f::LC_METHOD_LEN);         // spill the lowercased length
        }
        Arch::X86_64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rax", f::METHOD_PTR); // reload the original method-name pointer
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdx", f::METHOD_LEN); // reload the original method-name length
            abi::emit_call_label(ctx.emitter, "__rt_strtolower");               // rax=lowercased method-name copy, rdx=length
            abi::emit_store_to_sp(ctx.emitter, "rax", f::LC_METHOD_PTR);        // spill the lowercased pointer
            abi::emit_store_to_sp(ctx.emitter, "rdx", f::LC_METHOD_LEN);        // spill the lowercased length
        }
    }

    // -- locate the class's method-table segment via the class_id-indexed index array --
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x5", f::CLASS_ID); // reload the resolved class_id
            abi::emit_symbol_address(ctx.emitter, "x6", "_reflect_method_index"); // index table base
            ctx.emitter.instruction(&format!("mov x7, #{}", INDEX_ROW_SIZE));   // index rows are INDEX_ROW_SIZE bytes wide
            ctx.emitter.instruction("mul x7, x5, x7");                          // byte offset = class_id * INDEX_ROW_SIZE
            ctx.emitter.instruction("add x6, x6, x7");                          // index entry address = base + offset
            ctx.emitter.instruction("ldr x7, [x6]");                            // segment start (row index)
            ctx.emitter.instruction("ldr x8, [x6, #8]");                        // segment count
            ctx.emitter
                .instruction(&format!("cbz x8, {}", method_not_found));         // zero-row segment -> no such method
            abi::emit_symbol_address(ctx.emitter, "x9", "_reflect_method_table"); // method-table base
            ctx.emitter.instruction(&format!("mov x10, #{}", METHOD_ROW_SIZE)); // method-table rows are METHOD_ROW_SIZE bytes wide
            ctx.emitter.instruction("mul x10, x7, x10");                        // byte offset = start * METHOD_ROW_SIZE
            ctx.emitter.instruction("add x9, x9, x10");                         // segment base address = table base + offset
            // -- binary search the segment for the lowercased method name --
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x0", f::LC_METHOD_PTR); // lowercased query pointer -> search arg0
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", f::LC_METHOD_LEN); // lowercased query length -> search arg1
            ctx.emitter.instruction("mov x2, x9");                              // segment base -> search arg2
            ctx.emitter.instruction("mov x3, x8");                              // segment count -> search arg3
            ctx.emitter.instruction(&format!("mov x4, #{}", METHOD_ROW_SIZE));  // method-table entry size -> search arg4
            abi::emit_call_label(ctx.emitter, "__rt_sorted_name_search");       // -> x0 = matched row pointer or 0
            ctx.emitter
                .instruction(&format!("cbz x0, {}", method_not_found));         // no matching method in this segment
            // -- decode the matched row: real_name(ptr,len) + modifiers (zero-extended u32) --
            ctx.emitter.instruction(&format!("ldr x5, [x0, #{}]", METHOD_ROW_REAL_NAME_PTR_OFFSET)); // row.real_name_ptr
            ctx.emitter.instruction(&format!("ldr x6, [x0, #{}]", METHOD_ROW_REAL_NAME_LEN_OFFSET)); // row.real_name_len
            ctx.emitter.instruction(&format!("ldr w7, [x0, #{}]", METHOD_ROW_MODIFIERS_OFFSET)); // row.modifiers (32-bit, zero-extends into x7)
            abi::emit_store_to_sp(ctx.emitter, "x5", f::REAL_NAME_PTR);         // spill real_name_ptr across the object allocation call
            abi::emit_store_to_sp(ctx.emitter, "x6", f::REAL_NAME_LEN);         // spill real_name_len
            abi::emit_store_to_sp(ctx.emitter, "x7", f::MODIFIERS);             // spill modifiers
        }
        Arch::X86_64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "r9", f::CLASS_ID); // reload the resolved class_id
            abi::emit_symbol_address(ctx.emitter, "r10", "_reflect_method_index"); // index table base
            ctx.emitter.instruction(&format!("imul r9, r9, {}", INDEX_ROW_SIZE)); // byte offset = class_id * INDEX_ROW_SIZE
            ctx.emitter.instruction("add r10, r9");                             // index entry address = base + offset
            ctx.emitter.instruction("mov r11, QWORD PTR [r10]");                // segment start (row index)
            ctx.emitter.instruction("mov r9, QWORD PTR [r10 + 8]");             // segment count
            ctx.emitter.instruction("test r9, r9");                             // zero-row segment?
            ctx.emitter.instruction(&format!("jz {}", method_not_found));       // -> no such method
            abi::emit_symbol_address(ctx.emitter, "r10", "_reflect_method_table"); // method-table base
            ctx.emitter.instruction(&format!("imul r11, r11, {}", METHOD_ROW_SIZE)); // byte offset = start * METHOD_ROW_SIZE
            ctx.emitter.instruction("add r10, r11");                            // segment base address = table base + offset
            // -- binary search the segment for the lowercased method name --
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", f::LC_METHOD_PTR); // lowercased query pointer -> search arg0
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rsi", f::LC_METHOD_LEN); // lowercased query length -> search arg1
            ctx.emitter.instruction("mov rdx, r10");                            // segment base -> search arg2
            ctx.emitter.instruction("mov rcx, r9");                             // segment count -> search arg3
            ctx.emitter.instruction(&format!("mov r8, {}", METHOD_ROW_SIZE));   // method-table entry size -> search arg4
            abi::emit_call_label(ctx.emitter, "__rt_sorted_name_search");       // -> rax = matched row pointer or 0
            ctx.emitter.instruction("test rax, rax");                           // did the search return a matching entry?
            ctx.emitter.instruction(&format!("jz {}", method_not_found));       // no matching method in this segment
            // -- decode the matched row: real_name(ptr,len) + modifiers (zero-extended u32) --
            ctx.emitter.instruction(&format!("mov r9, QWORD PTR [rax + {}]", METHOD_ROW_REAL_NAME_PTR_OFFSET)); // row.real_name_ptr
            ctx.emitter.instruction(&format!("mov r10, QWORD PTR [rax + {}]", METHOD_ROW_REAL_NAME_LEN_OFFSET)); // row.real_name_len
            ctx.emitter.instruction(&format!("mov r11d, DWORD PTR [rax + {}]", METHOD_ROW_MODIFIERS_OFFSET)); // row.modifiers (32-bit, zero-extends into r11)
            abi::emit_store_to_sp(ctx.emitter, "r9", f::REAL_NAME_PTR);         // spill real_name_ptr across the object allocation call
            abi::emit_store_to_sp(ctx.emitter, "r10", f::REAL_NAME_LEN);        // spill real_name_len
            abi::emit_store_to_sp(ctx.emitter, "r11", f::MODIFIERS);            // spill modifiers
        }
    }

    // -- construct the ReflectionMethod shell object and bake its metadata slots --
    let (class_id, property_count, markers, modifiers_off, name_off) = {
        let ci = ctx
            .module
            .class_infos
            .get("ReflectionMethod")
            .ok_or_else(|| CodegenIrError::unsupported("unknown class ReflectionMethod"))?;
        let off = |n: &str| -> Result<usize> {
            ci.property_offsets
                .get(n)
                .copied()
                .ok_or_else(|| CodegenIrError::missing_entry("property offset", 0))
        };
        (
            ci.class_id,
            ci.properties.len(),
            super::uninitialized_property_marker_offsets(ci),
            off("__modifiers")?,
            off("__name")?,
        )
    };
    super::emit_object_allocation(ctx, class_id, property_count, false, &markers, &[])?;
    emit_reflection_int_property_from_frame(ctx, f::MODIFIERS, modifiers_off, modifiers_off + 8);
    emit_reflection_string_property_from_frame(ctx, f::REAL_NAME_PTR, f::REAL_NAME_LEN, name_off, name_off + 8);
    emit_reflection_attrs_property(ctx, "ReflectionMethod", &[], &[])?;
    abi::emit_release_temporary_stack(ctx.emitter, f::SIZE);
    emit_epilogue(ctx);

    ctx.emitter.label(&class_not_found);
    emit_reflect_exception_throw(
        ctx,
        b"Class \"",
        f::CLASS_PTR,
        f::CLASS_LEN,
        b"",
        None,
        b"\" does not exist",
    )?;

    ctx.emitter.label(&method_not_found);
    emit_reflect_exception_throw(
        ctx,
        b"Method ",
        f::CLASS_PTR,
        f::CLASS_LEN,
        b"::",
        Some((f::METHOD_PTR, f::METHOD_LEN)),
        b"() does not exist",
    )?;
    Ok(())
}

/// Emits `_elephc_reflect_property_new_dynamic`: resolves `(class_ptr, class_len, property_ptr,
/// property_len)` to a `ReflectionProperty` shell object, or throws a catchable
/// `\ReflectionException` (php -n verified messages: `Class "NAME" does not exist"` / `Property
/// CLASS::$NAME does not exist`). Property names are case-SENSITIVE — no lowercasing.
fn emit_property_dispatcher(ctx: &mut FunctionContext<'_>) -> Result<()> {
    use property_frame as f;
    ctx.emitter.blank();
    ctx.emitter
        .comment("--- reflection: dynamic ReflectionProperty(class, property) construction dispatch ---");
    ctx.emitter.label_global(PROPERTY_DISPATCH_LABEL);
    emit_prologue(ctx);
    abi::emit_reserve_temporary_stack(ctx.emitter, f::SIZE);

    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_store_to_sp(ctx.emitter, "x0", f::CLASS_PTR);             // spill the original class-name pointer
            abi::emit_store_to_sp(ctx.emitter, "x1", f::CLASS_LEN);             // spill the original class-name length
            abi::emit_store_to_sp(ctx.emitter, "x2", f::PROPERTY_PTR);          // spill the original property-name pointer (EXACT case, never lowercased)
            abi::emit_store_to_sp(ctx.emitter, "x3", f::PROPERTY_LEN);          // spill the original property-name length
        }
        Arch::X86_64 => {
            abi::emit_store_to_sp(ctx.emitter, "rdi", f::CLASS_PTR);            // spill the original class-name pointer
            abi::emit_store_to_sp(ctx.emitter, "rsi", f::CLASS_LEN);            // spill the original class-name length
            abi::emit_store_to_sp(ctx.emitter, "rdx", f::PROPERTY_PTR);         // spill the original property-name pointer (EXACT case, never lowercased)
            abi::emit_store_to_sp(ctx.emitter, "rcx", f::PROPERTY_LEN);         // spill the original property-name length
        }
    }

    let class_not_found = format!("{}_class_not_found", PROPERTY_DISPATCH_LABEL);
    let property_not_found = format!("{}_property_not_found", PROPERTY_DISPATCH_LABEL);
    emit_resolve_class_id(ctx, f::CLASS_PTR, f::CLASS_LEN, f::CLASS_ID, &class_not_found);

    // -- locate the class's property-table segment via the class_id-indexed index array --
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x5", f::CLASS_ID); // reload the resolved class_id
            abi::emit_symbol_address(ctx.emitter, "x6", "_reflect_property_index"); // index table base
            ctx.emitter.instruction(&format!("mov x7, #{}", INDEX_ROW_SIZE));   // index rows are INDEX_ROW_SIZE bytes wide
            ctx.emitter.instruction("mul x7, x5, x7");                          // byte offset = class_id * INDEX_ROW_SIZE
            ctx.emitter.instruction("add x6, x6, x7");                          // index entry address = base + offset
            ctx.emitter.instruction("ldr x7, [x6]");                            // segment start (row index)
            ctx.emitter.instruction("ldr x8, [x6, #8]");                        // segment count
            ctx.emitter
                .instruction(&format!("cbz x8, {}", property_not_found));       // zero-row segment -> no such property
            abi::emit_symbol_address(ctx.emitter, "x9", "_reflect_property_table"); // property-table base
            ctx.emitter.instruction(&format!("mov x10, #{}", PROPERTY_ROW_SIZE)); // property-table rows are PROPERTY_ROW_SIZE bytes wide
            ctx.emitter.instruction("mul x10, x7, x10");                        // byte offset = start * PROPERTY_ROW_SIZE
            ctx.emitter.instruction("add x9, x9, x10");                         // segment base address = table base + offset
            // -- binary search the segment for the EXACT-case property name --
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x0", f::PROPERTY_PTR); // query pointer -> search arg0
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", f::PROPERTY_LEN); // query length -> search arg1
            ctx.emitter.instruction("mov x2, x9");                              // segment base -> search arg2
            ctx.emitter.instruction("mov x3, x8");                              // segment count -> search arg3
            ctx.emitter.instruction(&format!("mov x4, #{}", PROPERTY_ROW_SIZE)); // property-table entry size -> search arg4
            abi::emit_call_label(ctx.emitter, "__rt_sorted_name_search");       // -> x0 = matched row pointer or 0
            ctx.emitter
                .instruction(&format!("cbz x0, {}", property_not_found));       // no matching property in this segment
            ctx.emitter.instruction(&format!("ldr w5, [x0, #{}]", PROPERTY_ROW_MODIFIERS_OFFSET)); // row.modifiers (32-bit, zero-extends into x5)
            abi::emit_store_to_sp(ctx.emitter, "x5", f::MODIFIERS);             // spill modifiers across the object allocation call
        }
        Arch::X86_64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "r9", f::CLASS_ID); // reload the resolved class_id
            abi::emit_symbol_address(ctx.emitter, "r10", "_reflect_property_index"); // index table base
            ctx.emitter.instruction(&format!("imul r9, r9, {}", INDEX_ROW_SIZE)); // byte offset = class_id * INDEX_ROW_SIZE
            ctx.emitter.instruction("add r10, r9");                             // index entry address = base + offset
            ctx.emitter.instruction("mov r11, QWORD PTR [r10]");                // segment start (row index)
            ctx.emitter.instruction("mov r9, QWORD PTR [r10 + 8]");             // segment count
            ctx.emitter.instruction("test r9, r9");                             // zero-row segment?
            ctx.emitter.instruction(&format!("jz {}", property_not_found));     // -> no such property
            abi::emit_symbol_address(ctx.emitter, "r10", "_reflect_property_table"); // property-table base
            ctx.emitter.instruction(&format!("imul r11, r11, {}", PROPERTY_ROW_SIZE)); // byte offset = start * PROPERTY_ROW_SIZE
            ctx.emitter.instruction("add r10, r11");                            // segment base address = table base + offset
            // -- binary search the segment for the EXACT-case property name --
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", f::PROPERTY_PTR); // query pointer -> search arg0
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rsi", f::PROPERTY_LEN); // query length -> search arg1
            ctx.emitter.instruction("mov rdx, r10");                            // segment base -> search arg2
            ctx.emitter.instruction("mov rcx, r9");                             // segment count -> search arg3
            ctx.emitter.instruction(&format!("mov r8, {}", PROPERTY_ROW_SIZE)); // property-table entry size -> search arg4
            abi::emit_call_label(ctx.emitter, "__rt_sorted_name_search");       // -> rax = matched row pointer or 0
            ctx.emitter.instruction("test rax, rax");                           // did the search return a matching entry?
            ctx.emitter.instruction(&format!("jz {}", property_not_found));     // no matching property in this segment
            ctx.emitter.instruction(&format!("mov r9d, DWORD PTR [rax + {}]", PROPERTY_ROW_MODIFIERS_OFFSET)); // row.modifiers (32-bit, zero-extends into r9)
            abi::emit_store_to_sp(ctx.emitter, "r9", f::MODIFIERS);             // spill modifiers across the object allocation call
        }
    }

    // -- construct the ReflectionProperty shell object and bake its metadata slots --
    let (class_id, property_count, markers, modifiers_off, name_off) = {
        let ci = ctx
            .module
            .class_infos
            .get("ReflectionProperty")
            .ok_or_else(|| CodegenIrError::unsupported("unknown class ReflectionProperty"))?;
        let off = |n: &str| -> Result<usize> {
            ci.property_offsets
                .get(n)
                .copied()
                .ok_or_else(|| CodegenIrError::missing_entry("property offset", 0))
        };
        (
            ci.class_id,
            ci.properties.len(),
            super::uninitialized_property_marker_offsets(ci),
            off("__modifiers")?,
            off("__name")?,
        )
    };
    super::emit_object_allocation(ctx, class_id, property_count, false, &markers, &[])?;
    emit_reflection_int_property_from_frame(ctx, f::MODIFIERS, modifiers_off, modifiers_off + 8);
    // The property's display name is always the EXACT query bytes: a successful exact-case
    // binary search hit means `row.name == query bytes`, so the original (still on the frame)
    // query is baked directly instead of re-reading the row.
    emit_reflection_string_property_from_frame(ctx, f::PROPERTY_PTR, f::PROPERTY_LEN, name_off, name_off + 8);
    emit_reflection_attrs_property(ctx, "ReflectionProperty", &[], &[])?;
    abi::emit_release_temporary_stack(ctx.emitter, f::SIZE);
    emit_epilogue(ctx);

    ctx.emitter.label(&class_not_found);
    emit_reflect_exception_throw(
        ctx,
        b"Class \"",
        f::CLASS_PTR,
        f::CLASS_LEN,
        b"",
        None,
        b"\" does not exist",
    )?;

    ctx.emitter.label(&property_not_found);
    emit_reflect_exception_throw(
        ctx,
        b"Property ",
        f::CLASS_PTR,
        f::CLASS_LEN,
        b"::$",
        Some((f::PROPERTY_PTR, f::PROPERTY_LEN)),
        b" does not exist",
    )?;
    Ok(())
}

/// Stores a RUNTIME modifiers value (already resolved into the frame at `value_off`, a
/// zero-extended 32-bit field) into the current object-under-construction's property slot. No
/// call happens, so the object pointer (already in the ABI int-result register from the
/// preceding `emit_object_allocation`) needs no push/pop preservation — mirrors
/// `super::reflection::emit_reflection_int_property`'s immediate-value sibling.
fn emit_reflection_int_property_from_frame(ctx: &mut FunctionContext<'_>, value_off: usize, low_offset: usize, high_offset: usize) {
    let object_reg = abi::int_result_reg(ctx.emitter);
    let scratch = abi::secondary_scratch_reg(ctx.emitter);
    abi::emit_load_temporary_stack_slot(ctx.emitter, scratch, value_off);       // reload the runtime modifiers value from the reserved frame
    abi::emit_store_to_address(ctx.emitter, scratch, object_reg, low_offset);   // store the modifiers value into the property's low word
    abi::emit_store_zero_to_address(ctx.emitter, object_reg, high_offset);      // the property's high word stays zero (plain int, not Mixed)
}

/// Persists a RUNTIME (ptr, len) string value (already resolved into the frame at
/// `ptr_off`/`len_off`) and stores it into the current object-under-construction's property slot.
/// Mirrors `super::reflection::emit_reflection_string_property`'s compile-time-immediate sibling,
/// but reads its source bytes from the frame instead of a Rust `&str`, and preserves the object
/// pointer across the `__rt_str_persist` call via the SAME push/pop-around-a-call convention.
fn emit_reflection_string_property_from_frame(ctx: &mut FunctionContext<'_>, ptr_off: usize, len_off: usize, low_offset: usize, high_offset: usize) {
    let object_reg = abi::symbol_scratch_reg(ctx.emitter);
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, result_reg);                                // park the object-under-construction pointer across the persist call
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", ptr_off + 16); // reload the row's real-name pointer (offset shifts by the just-pushed 16 bytes)
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x2", len_off + 16); // reload the row's real-name length
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");              // own a heap copy of the name bytes -> x1=ptr, x2=len
            abi::emit_pop_reg(ctx.emitter, object_reg);                         // restore the object pointer into a scratch register
            abi::emit_store_to_address(ctx.emitter, "x1", object_reg, low_offset); // store the persisted name pointer
            abi::emit_store_to_address(ctx.emitter, "x2", object_reg, high_offset); // store the persisted name length
        }
        Arch::X86_64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rax", ptr_off + 16); // reload the row's real-name pointer (offset shifts by the just-pushed 16 bytes)
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdx", len_off + 16); // reload the row's real-name length
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");              // own a heap copy of the name bytes -> rax=ptr, rdx=len
            abi::emit_pop_reg(ctx.emitter, object_reg);                         // restore the object pointer into a scratch register
            abi::emit_store_to_address(ctx.emitter, "rax", object_reg, low_offset); // store the persisted name pointer
            abi::emit_store_to_address(ctx.emitter, "rdx", object_reg, high_offset); // store the persisted name length
        }
    }
    abi::emit_push_reg(ctx.emitter, object_reg);                                // put the object pointer back...
    abi::emit_pop_reg(ctx.emitter, result_reg);                                 // ...into the ABI int-result register
}
